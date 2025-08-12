from http.server import BaseHTTPRequestHandler, HTTPServer

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(b"Hello from Python!")

if __name__ == "__main__":
    with HTTPServer(("localhost", 8000), Handler) as httpd:
        print("Serving on http://localhost:8000")
        httpd.serve_forever()
