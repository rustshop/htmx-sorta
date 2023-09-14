function installSortable(element) {
      new Sortable(element, {
          animation: 150,
          ghostClass: 'blue-background-class',
          handle: '.handle',
          draggable: '.draggable',
          onEnd: function (evt) {
            const to = evt.to;
            const eventDstElement = to;
            const prevElement = to.children[evt.newIndex - 1];
            const prevElementIsItem = prevElement?.classList.contains('draggable');
            const prev = prevElementIsItem ? prevElement?.id : undefined;
            const curr = to.children[evt.newIndex]?.id;
            const next = to.children[evt.newIndex + 1]?.id;
            eventDstElement.setAttribute("hx-vals", JSON.stringify({ prev, curr, next }));
            eventDstElement.dispatchEvent(new Event("changed"));
            eventDstElement.setAttribute("hx-vals", "");
          }
      });
}

function installSortableInChildren(element) {
    var sortables = element.querySelectorAll(".sortable");
    if (element.classList.contains('sortable')) {
        installSortable(element);
    }
    for (var i = 0; i < sortables.length; i++) {
      var sortable = sortables[i];
      installSortable(sortable);
    }
}

htmx.onLoad(function(content) {
  installSortableInChildren(content);
})

document.body.addEventListener('htmx:afterSwap', function(evt) {
  installSortableInChildren(evt.target);
});
