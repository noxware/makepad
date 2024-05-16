- Non breaking bevy like ref apis.
  = WidgetRef more like `WidgetHandle` to diferenciate from `Ref<Widget>`.
- `self.view.widget(...).into_ref::<Button>().clicked(...)` allows `Ref<Button>` to be passed around.
  - Into consuming to avoid needing to keey the widget handle in scope for ownership issues.
- `self.view.widget(...).into_ref_mut::<Button>().set_text(...)` allows `RefMut<Button>` to be passed around.
- In a different branch do more breaking changes like:
  - Remove macro generation of widgets.
  - Remove duplicate functions.
    - Idk what to do with the image load stuff.
