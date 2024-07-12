# Objetivo

- Permitir que un widget reaccione (se actualice y/o se redibuje) cuando datos externos a el
cambian.
- Esto, sin tener que dispatchear una accion especifica para avisarle a un widget especifico, sino,
  dispatcheando una notificacion general para avisar que X dato cambio sin importar quien lo cambio.
- En lo posible, que el widget que origino el cambio no tenga que avisar manualmente, y que en cambio el aviso
  se haga desde dentro del store.
- La implementacion debe ser minimalista para que sea mas facil de integrar con el store de Moxin de ser querido.


# Implicaciones de lo requerido

Para permitir que el store avise, debe tener accesso al context cuando se ejecute una mutacion.
Por este motivo, hay que pasar el `cx` al ejecutar una funcion mutante.

Ejemplo en `ui.rs`:

```rust
fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
    let app_state = scope.data.get_mut::<AppState>().unwrap();

    if self.button(id!(increment)).clicked(actions) {
        app_state.increment_counter(cx);
    }

    if self.button(id!(decrement)).clicked(actions) {
        app_state.decrement_counter(cx);
    }
}
```

Como el store dispatchea una accion cuando la mutacion ocurre, esta se puede detectar escuchando los
eventos en makepad.

En esta implementacion, se intento llegar a una solucion reusable y minimalista. Por lo que se creo la abstraccion
`Subject`, un wrapper pequeño que al llamar a su metodo `set` (que requiere `cx`) dispatchea la notificacion de cambio,
y que te da un metodo `changed` para saber si el valor cambio.

Ejemplo en `ui.rs`:

```rust
fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
    let app_state = scope.data.get_mut::<AppState>().unwrap();

    if app_state.counter.changed(event) || !self.initialized {
        self.label(id!(value))
            .set_text(&app_state.counter.get().to_string());
        self.redraw(cx);
    }

    self.initialized = true;

    self.widget_match_event(cx, event, scope);
    self.deref.handle_event(cx, event, scope);
}
```

> Notese que no hice a `Subject` responsable del lifecycle del widget y por ende detectar la inicializacion aparte es necesario.
> Se me oucrren un par de formas de que `Subject` colabore con el lifecycle pero por ahora centremonos solo en cumplir los objetivos antes planteados.



# Notas del codigo

- Por supuesto un contador no necesita ni tener estado a nivel de app, ni necesita reactividad pero
  este ejemplo es el defacto para este tipo de cosas.
- `app_state` = `store` de moxin.
- `subject.rs` es donde esta el codigo de `Subject`.
- `app_state.rs` usa subject en uno de sus fields mutables.
  - En Moxin, el store podria tener `Subject<Vec<Chat>>` por ejemplo.
- `ui.rs` es el widget raiz de la aplicacion (y el unico). Es donde esta el contador y donde se expone bien la idea.
- `app.rs` simplemente mantiene y pasa `app_state`.
