document.addEventListener('htmx:sendError', function(event) {
  let errorMessage = document.getElementById('htmx-send-error');
  errorMessage.classList.remove('hidden');

  requestAnimationFrame(() => {
    errorMessage.classList.add('showing');
  }); 

  document.getElementById("gray-out-page").classList.remove("hidden");
  document.getElementById("gray-out-page").classList.add("showing");

  setTimeout(() => {
    errorMessage.classList.remove('showing');
    setTimeout(() => {
      errorMessage.classList.add('hidden');
    }, 500);
  }, 5000);
});
