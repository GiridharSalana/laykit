// Make the book title clickable and link to GitHub repo
document.addEventListener('DOMContentLoaded', function() {
    const menuTitle = document.querySelector('.menu-title');
    if (menuTitle) {
        // Wrap the title text in a link
        const titleText = menuTitle.textContent;
        menuTitle.innerHTML = '<a href="https://github.com/giridharsalana/laykit" target="_blank" rel="noopener noreferrer">' + titleText + '</a>';
    }
    
    // Start with sidebar collapsed
    const html = document.querySelector('html');
    if (html && !localStorage.getItem('mdbook-sidebar')) {
        // Only hide on first visit or if user hasn't set a preference
        html.classList.remove('sidebar-visible');
    }
});
