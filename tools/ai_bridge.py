import socket
from llama_cpp import Llama
import unicodedata

# Configuration
MODEL_PATH = "SmolLM2-135M-Instruct-Q8_0.gguf"
HOST = '127.0.0.1'
PORT = 1234

print(f"--- PONT IA STABILISE POUR JC-OS ---")
# n_ctx=1024 pour un meilleur suivi des conversations longues
llm = Llama(model_path=MODEL_PATH, n_ctx=1024, verbose=False)

def clean_text(text):
    """Normalise le texte pour le driver VGA (ASCII uniquement)"""
    nfkd_form = unicodedata.normalize('NFKD', text)
    return "".join([c for c in nfkd_form if not unicodedata.combining(c)])

def start_bridge():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        # Permet de relancer le script immédiatement sans erreur de port
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT))
        s.listen()
        print(f"--- JC-AI PRET SUR LE PORT {PORT} ---")
        
        while True:
            conn, addr = s.accept()
            with conn:
                print(f"[KERNEL CONNECTE]")
                session_data = ""
                while True:
                    data = conn.recv(1024)
                    if not data: break
                    
                    fragment = data.decode('utf-8', errors='ignore')
                    session_data += fragment

                    # SYNCHRONISATION : On attend le '\n' envoyé par serial_println!
                    if "AI_REQ:" in session_data and session_data.endswith('\n'):
                        query = session_data.split("AI_REQ:")[-1].strip()
                        
                        if not query:
                            session_data = ""
                            continue

                        print(f"\n[IA] Question complete recue : {query}")

                        # Génération avec température basse pour la précision (ton réglage 0.1)
                        output = llm.create_chat_completion(
                            messages=[
                                {
                                    "role": "system", 
                                    "content": "You are JC-AI, a helpful assistant for JC-OS. Be precise and concise."
                                },
                                {"role": "user", "content": query}
                            ],
                            max_tokens=100,
                            temperature=0.1,  # Stabilité maximale
                            top_p=0.9,
                            repeat_penalty=1.2 # Évite les bégaiements du modèle
                        )

                        response = output['choices'][0]['message']['content']
                        
                        # Nettoyage pour affichage sur une seule ligne VGA
                        final_reply = clean_text(response).replace('\n', ' ').strip()
                        
                        print(f"[IA] Reponse : {final_reply}")
                        
                        # Envoi avec \n pour débloquer le read_line() du Kernel
                        conn.sendall(f"{final_reply}\n".encode('utf-8'))
                        
                        session_data = "" # Reset pour la prochaine commande

if __name__ == "__main__":
    start_bridge()