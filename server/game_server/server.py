import socket
import threading

def handle_data(sock, data):
    print("Received:", data)
    resp = None
    if data == b'JMUS_CHECK\r':
        resp = b'VALID'
    if data == b'US\r':
        #iV: inventory list
        #lg: Language
        #mm: mute
        # cm: indexes for body part models
        ## 1: gender
        ## 2: hair style
        ## 3: ??
        ## 4: ??
        ## 5: ??
        ## 6: ??
        ## 7: ??
        ## 8: skin color
        ## 9: hair color
        ## 10: ??
        # cm = "111011131320232"
        cm = "241111112111111"
        #ig: gem inventory (not included)
        resp = f'U [#uid: 1, #dn: "bawoosette", #iV: [], #lg: "en", #mm: 0, #cm: "{cm}", #uc: [], #uh: [], #bl: [], #bu: [], #cb: "", #pa: [], #pp: 0, #ppnew: 0, #gt: 0, #ga: 0, #hs: []]'
        resp = resp.encode("utf-8")
    # if data == b'CT\r\rGET'
    # if data.startswith(b'MU'):
    #     resp = b'MU []'
    if resp is not None:
        print("Responding:", resp)
        sock.send(resp)

def handle_client(client_socket, client_address):
    print(f"Connection from {client_address}")
    
    # Receive data from the client
    while True:
        data = client_socket.recv(1024)
        if not data:
            break
        handle_data(client_socket, data)
    
    # Close the client socket
    client_socket.close()

def listen_on_port(port):
    try:
        # Create a socket object
        server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        
        # Bind the socket to a specific address and port
        server_socket.bind(('0.0.0.0', port))
        
        # Listen for incoming connections
        server_socket.listen(5)
        
        print(f"Listening on port {port}...")
        
        while True:
            # Wait for a client connection
            client_socket, client_address = server_socket.accept()
            
            # Create a new thread to handle the client
            client_thread = threading.Thread(target=handle_client, args=(client_socket, client_address))
            client_thread.start()
    except KeyboardInterrupt:
        server_socket.close()

    server_socket.close()

if __name__ == "__main__":
    listen_on_port(7158)
