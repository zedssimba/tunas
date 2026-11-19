import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

// Block the browser's own drop-to-open behaviour; files are handled by the chat.
window.addEventListener('dragover', (e) => e.preventDefault());
window.addEventListener('drop', (e) => e.preventDefault());
// Right-click menus and Ctrl+R reload are not useful in the app.
window.addEventListener('keydown', (e) => {
  if ((e.ctrlKey && (e.key === 'r' || e.key === 'R')) || e.key === 'F5') e.preventDefault();
});

export default mount(App, { target: document.getElementById('app') });
