import socket
import unicodedata
import os
from dotenv import load_dotenv
from openai import OpenAI
from pathlib import Path

# Charge le .env du dossier courant
load_dotenv(Path(__file__).parent / ".env")

# Vérification clé GROQ
api_key = os.getenv("GROQ_API_KEY")
if not api_key:
    raise RuntimeError("GROQ_API_KEY not found in environment")

# Client Groq (compatible OpenAI)
client = OpenAI(
    api_key=api_key,
    base_url="https://api.groq.com/openai/v1"
)

# Configuration réseau
HOST = '127.0.0.1'
PORT = 1234

print("--- PONT CLOUD GROQ POUR JC-OS : INITIALISATION ---")

def clean_text(text: str) -> str:
    """Indispensable pour ton driver VGA Rust"""
    nfkd_form = unicodedata.normalize('NFKD', text)
    return "".join([c for c in nfkd_form if not unicodedata.combining(c)])

def start_bridge():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT))
        s.listen()
        print(f"--- JC-AI (GROQ) PRET SUR LE PORT {PORT} ---")
        
        while True:
            conn, addr = s.accept()
            with conn:
                print("[KERNEL CONNECTE VIA CLOUD]")
                session_data = ""

                while True:
                    data = conn.recv(1024)
                    if not data:
                        break
                    
                    fragment = data.decode('utf-8', errors='ignore')
                    session_data += fragment

                    if "AI_REQ:" in session_data and session_data.endswith('\n'):
                        query = session_data.split("AI_REQ:")[-1].strip()
                        
                        if not query:
                            session_data = ""
                            continue

                        print(f"\n[CLOUD QUERY] : {query}")

                        try:
                            completion = client.chat.completions.create(
                                model="llama-3.1-8b-instant",  # modèle supporté Groq
                                messages=[
                                    {
                                        "role": "system", 
                                        "content": "You are JC-AI running inside a custom Rust kernel. Be extremely concise, max 15 words."
                                    },
                                    {"role": "user", "content": query}
                                ],
                                max_tokens=50
                            )

                            response = completion.choices[0].message.content
                            final_reply = clean_text(response).replace('\n', ' ').strip()
                            
                            print(f"[IA CLOUD] : {final_reply}")
                            conn.sendall(f"{final_reply}\n".encode('utf-8'))
                        
                        except Exception as e:
                            error_msg = f"Erreur API: {str(e)}"
                            print(error_msg)
                            conn.sendall(f"{error_msg}\n".encode('utf-8'))
                        
                        session_data = ""

if __name__ == "__main__":
    start_bridge()
