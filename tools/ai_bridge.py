import socket
from llama_cpp import Llama
import unicodedata

# Configuration
MODEL_PATH = "SmolLM2-135M-Instruct-Q8_0.gguf"
HOST = '127.0.0.1'
PORT = 1234

print(f"--- PONT IA PYTHON POUR JC-OS : CHARGEMENT ---")
# Initialisation du modèle (n_ctx=512 pour être léger)
llm = Llama(model_path=MODEL_PATH, n_ctx=512, verbose=False)

def clean_text(text):
    """Enlève les accents pour le driver VGA de JC-OS"""
    nfkd_form = unicodedata.normalize('NFKD', text)
    return "".join([c for c in nfkd_form if not unicodedata.combining(c)])

def start_bridge():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((HOST, PORT))
        s.listen()
        print(f"--- PRET A RECEPTONNER SUR LE PORT {PORT} ---")
        
        while True:
            conn, addr = s.accept()
            with conn:
                print(f"[KERNEL CONNECTE]")
                session_data = ""
                while True:
                    data = conn.recv(1024)
                    if not data: break
                    
                    fragment = data.decode('utf-8', errors='ignore')
                    print(fragment, end='', flush=True)
                    session_data += fragment

                    if "AI_REQ:" in session_data:
                        # Extraction de la question
                        query = session_data.split("AI_REQ:")[-1].strip()
                        print(f"\n[IA] Analyse de : {query}")

                        # Génération ChatML
                        output = llm.create_chat_completion(
                            messages=[
                                {"role": "system", "content": "You are JC-AI. Be brief."},
                                {"role": "user", "content": query}
                            ],
                            max_tokens=50
                        )

                        response = output['choices'][0]['message']['content']
                        print(f"[IA] Reponse : {response}")

                        # Nettoyage et envoi
                        final_reply = clean_text(response).replace('\n', ' ')
                        conn.sendall(f"{final_reply}\n".encode('utf-8'))
                        
                        session_data = "" # Reset pour la suite

if __name__ == "__main__":
    start_bridge()