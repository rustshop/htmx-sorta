document.addEventListener('htmx:sendError', function(event) {
  document.getElementById("gray-out-page").classList.remove("send-error-hidden");
  document.getElementById("gray-out-page").classList.add("send-error-showing");
});
