document.getElementById("greet-btn").addEventListener("click", async () => {
    const response = await fetch("/api/greet");
    const data = await response.json();
    document.getElementById("greet-response").textContent = data.message;
});
