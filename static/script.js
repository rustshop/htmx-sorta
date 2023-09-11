htmx.onLoad(function(content) {
    var sortables = content.querySelectorAll(".sortable");
    for (var i = 0; i < sortables.length; i++) {
      var sortable = sortables[i];
      new Sortable(sortable, {
          animation: 150,
          ghostClass: 'blue-background-class',
          handle: '.handle',
          onEnd: function (evt) {
            const to = evt.to;
            const toParent = to.parentElement;
            const prev = to.children[evt.newIndex - 1]?.id;
            const curr = to.children[evt.newIndex]?.id;
            const next = to.children[evt.newIndex + 1]?.id;
            toParent.setAttribute("hx-vals", JSON.stringify({ prev, curr, next }));
            toParent.dispatchEvent(new Event("changed"));
            toParent.setAttribute("hx-vals", "");
          }
      });
    }
})
