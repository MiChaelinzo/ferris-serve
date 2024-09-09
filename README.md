# ferris-serve
A foundation for a robust and feature-rich file server. Currently supports serving static files, file uploads, user authentication, and more etc.

**How to Run:**

1.  **Save the code** as `main.rs`.
2.  **Create a directory** (e.g., "public") with an `index.html` file.
3.  **Build and run:**

    ```bash
    cargo run 8080 public 
    ```

    (Replace `8080` and "public" if needed)

4.  **Test:**
    - Access `http://127.0.0.1:8080/` in your browser to see the `index.html` file.
    - Try uploading a file using a tool like `curl`. You will need to provide Basic authentication credentials in the header.
