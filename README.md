# ferris-serve
A foundation for a robust and feature-rich file server. Currently supports serving static files with plans to add file uploads, user authentication, and more.

**How to Run:**

1.  **Save:**  Save the code as `main.rs`.
2.  **Create Directory:** Create a directory (e.g., "public") and place files you want to serve inside. For example, create an `index.html` file.
3.  **Run:**  In your terminal, navigate to the directory and run the server:

    ```bash
    cargo run 8080 public
    ```

    Replace `8080` with your desired port and "public" with your directory name.

4.  **Access:** Open `http://127.0.0.1:8080/` in your browser. 

