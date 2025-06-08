# CNCURSES

A Component Based Library for ncurses. The Library takes its inspiration from Flutter to declare the Components and React to update the State.
The motivation came as an update to the [rstop](https://github.com/alonot/rstop/) was needed. The project previously tried to decouple frontend logic, backend and the display mechanism in a very non-scalable manner.

## Interface

The basic Unit available as public to users is a trait `Component`.
The library provides two global functions : `run` and `use_state`

To create custom Component, one can implement this trait : `Component` and pass the root component to `run` function.

`run` : initializes the internal states and starts an infinite loop which check for checks as user interact with the app.

Library currently provide following `Base` Components which can be used to create Custom Components:

`View`: Can hold other Components i.e. an Array of components.
`Text`: holds a text.
`Button`: A wrapper over View with only one child (which can be another `Component`). This forces to give n onlick function.

`Component`:
Component have a `__call__` method which must be implemented by the Custom Component.
`__call__` expects `Arc<Mutex<dyn Component>>` as result. Wraping each component in `Arc<Mutex<>>` becomes too much of boiler plate.
Hence, Internally Library implement a `ComponentBuilder` trait on every type which implement Component. One can import this trait and call
`.build()` on the Component object to convert it to the required format.

Example:

```rust
View::new(
    vec![],
    CSSStyle {
        ..Default::default()
    },
)      // ----->  will return View object which implment Component trait
.build() // ---->  converts the View object to Arc<Mutex<dyn Component>>
```

#### Styling

For declaring `Base` Components, you need to pass the `CSSStyles` Associated with it.
One can use `..Default::default()` to keep other styles to their default.
Each `Base` Component also provides with a `new_style_vec` version which uses a list of `STYLE` enum to assign styles to the Component.
This method took too much boilerplate, hence `CSSStyle` class was created.


###### Here is the default of all the Styles:

| Property           | Default Value                  | Type              |
| ------------------ | ------------------------------ | ----------------- |
| `height`           | `DIMEN::INT(-1)` (FIT_CONTENT) | DIMEN             |
| `width`            | `DIMEN::INT(-1)` (FIT_CONTENT) | DIMEN             |
| `top`              | `DIMEN::INT(0)`                | DIMEN             |
| `left`             | `DIMEN::INT(0)`                | DIMEN             |
| `paddingleft`      | `DIMEN::INT(0)`                | DIMEN             |
| `paddingtop`       | `DIMEN::INT(0)`                | DIMEN             |
| `paddingright`     | `DIMEN::INT(0)`                | DIMEN             |
| `paddingbottom`    | `DIMEN::INT(0)`                | DIMEN             |
| `marginleft`       | `DIMEN::INT(0)`                | DIMEN             |
| `margintop`        | `DIMEN::INT(0)`                | DIMEN             |
| `marginright`      | `DIMEN::INT(0)`                | DIMEN             |
| `marginbottom`     | `DIMEN::INT(0)`                | DIMEN             |
| `border`           | `0`                            | i32               |
| `border_color`     | `-1`                           | i16               |
| `color`            | `-1`                           | i16               |
| `background_color` | `-2`                           | i16               |
| `flex`             | `0`                            | u32               |
| `flex_direction`   | `FLEXDIRECTION::default()`     | FLEXDIRECTION     |
| `position`         | `POSITION::default()`          | POSITION          |
| `boxsizing`        | `BOXSIZING::default()`         | BOXSIZING         |
| `taborder`         | `-1`                           | i32               |
| `z_index`          | `0`                            | i32               |
| `render`           | `true`                         | bool              |
| `scroll`           | `OVERFLOWBEHAVIOUR::HIDDEN`    | OVERFLOWBEHAVIOUR |

###### Here is the format for the CSSStyle attributes:

| CSSStyle Field     | Format                                            | Example                                | Notes                            |
| ------------------ | ------------------------------------------------- | -------------------------------------- | -------------------------------- |
| `padding`          | Space-separated values: `"top bottom left right"` | `"10 5 10 5"` or `"10% 5% 10% 5%"`     | Parsed into 4 DIMEN values       |
| `margin`           | Space-separated values: `"top bottom left right"` | `"20 10 20 10"` or `"5% 2% 5% 2%"`     | Parsed into 4 DIMEN values       |
| `background_color` | Integer color code                                | `-1` (default/transparent)             | i16 value                        |
| `color`            | Integer color code                                | `-1` (default)                         | i16 value                        |
| `flex`             | Unsigned integer                                  | `0` (default), `1`, `2`, etc.          | u32 value                        |
| `flex_direction`   | String literal                                    | `"vertical"` or `"horizontal"`         | Parsed to FLEXDIRECTION enum     |
| `taborder`         | Integer                                           | `-1` (default/no tab order)            | i32 value                        |
| `border_color`     | Integer color code                                | `-1` (default)                         | i16 value                        |
| `position`         | String literal                                    | `"static"` or `"relative"`             | Parsed to POSITION enum          |
| `boxsizing`        | String literal                                    | `"border-box"` or `"content-box"`      | Parsed to BOXSIZING enum         |
| `border`           | Integer                                           | `0` (no border)                        | i32 value                        |
| `top`              | Dimension string                                  | `"10"` or `"50%"`                      | Parsed to DIMEN                  |
| `left`             | Dimension string                                  | `"0"` or `"25%"`                       | Parsed to DIMEN                  |
| `height`           | Dimension string                                  | `"100"` or `"auto"`                    | Parsed to DIMEN                  |
| `width`            | Dimension string                                  | `"200"` or `"100%"`                    | Parsed to DIMEN                  |
| `scroll`           | String literal                                    | `"scroll"`, `"visible"`, or `"hidden"` | Parsed to OVERFLOWBEHAVIOUR enum |
| `z_index`          | Integer                                           | `0` (default layer)                    | i32 value(Not Implemented yet)                        |

##### Dimension Format Notes:

- **Integer values**: Plain numbers like `"10"`, `"100"`, `"-1"`
- **Percentage values**: Floating point Numbers followed by `%` like `"50%"`, `"100%"`
- **Special constants**:
  - `FIT_CONTENT = -1` (content-based sizing)
- **Multi-dimension fields** (padding, margin): Must contain exactly 4 space-separated values

##### Validation Rules:

- **DIMEN::INT**: Must be >= -1 (FIT_CONTENT)
- **DIMEN::PERCENT**: Must be between 0.0 and 100.0 (converted to 0.0-1.0 internally)
- **Multi-dimension parsing**: Expects exactly 4 values, will panic if different count provided

The reason that `Base` components' new do not return `Arc<Mutex<>>` directly is that we can use this returned object like a builder to assign
event listeners like `onscroll`, `onclick`, etc
The mouse event listeners provide both `event capturing` and `event bubbling`.
Event listeners get an event object
one can stop the propogation of event using `event_object.stop_propogation`
and prevent default behaviour using `event_object.prevent_default`

#### Focus

One can use tab to focus on focusable element. An element can be made focusable by setting a non-negative `taborder` style property.
The main use and aim of this feature is in future when library enable support for `Forms`

#### Scrolling with Keyboard

- Whenever users clicks on any view the deepest child with overflow set to scroll becomes current `active` iview.
- On further interactions with keyboard UP, DOWN, RIGHT,LEFT, this currect `active` child's scroll behaviour is triggered.
- When an elements comes into focus through tab, and it is scrollable then current `active` is set to that same element.

## Fote Note

PS: I am really new to rust and may not have used the language at its fullest. I'am still working on this library, and testing of this library is not done as required.
I am trying to re-create [rstop](https://github.com/alonot/rstop/) project using this library. This would enable me to discover bugs as I go. Thank you.

Now, I'll try to explain some of the internal structure of the library. Will be update

## Internals

```

Important Structures

Public:

trait Component

Enum STYLE:

Private:

Fiber: Manages the internal state for each component.

IView: Internal View


```

#### SetState

- The closure returned by setState captures the current fiber reference and the hook(state) of that fiber it represent.
- Whenever a component's `__call__` is called, the callee sets the curr_fiber so that the useState either uses the already existing fiber's state or creates new state completely.

The Component structure is converted to a Tree of `Fiber`. Each `Fiber` contains a reference to `IView`(Short Form for Internal View). A single IView may link up to many `Fiber`.
The library looks up for changes in `Fiber` tree, and then updates and render the `IView` created through the `Fiber` tree.

#### Fiber

- Manages State. While Parsing the `Fiber` tree, `document.curr_fiber` is set to the current fiber and then the Component's `__call__` method of the Component.
- `SetState` expects curr_fiber to be set and uses it accordingly to either create a new State or return already present State.
- Contains Reference to the `IView` tree and updates/Creates the tree.

#### IView

- Manages the Actual Ncurses window.
- `__init__` : Calculate the dimensions of the Component this IVIew is representing.
- `__render__`: Actually render the the window. The child sends its ncurses' `Window` back to the parent which copies it to its window and destoys the child's one. In this way, only root `IView`'s is left.

## Rendering:

The IView tree contains the basic ncurses WINDOW to render the contents. If there is any change in IView tree (style.render = True) then first **init** is called on the IView tree root, which will in-turn call **init** on its child. Child returns (lines, cols) which is the lines and cols it took(may be given by the user or calculate by its own child).

The actual rendering is done by **render** method. It returns (topleftx, toplefty, lines, col) depicting the dirty rectangle, which has been rendered by itself or by its children. It is clear that on first call to **render** this area will eventually be the area of whole screen. Parent copies the window of child into itself using two information. 1. bounds given by children, 2. its own scroll position.

This copying may be more optimized by returning a list of non-overlapping bounds. However, the cost of multiple call to copywin() has to be taken to account then.

#### Internals:

- All texts will use **pad**

- In **init**(), if child has flex attributes then parent will set the dimensions as percentage for the children

- Flex direction, and top , bottom, left ,right all these play effect in **render**() function.

NOTE: Dimension -> Height/Width

- Each IView will generate a window according to its `content` Dimension, not its actual Dimension. In case of OVERFLOW set to VISIBLE, the `content` Dimension will be equal to include all its child's independent of the actual Dimension. The parent while rendering will use the actual Dimensions to decide where to place the child. However the actual render will be with help of child's `content` Dimension.

##### Placing Child:

- Parent will keep `top`, `left` pointer to know which child should be rendered. The `cursorx` and `cursory` will specify to the actual location on parent's screen. Child render box will be clipped according to the scroll view, if visibility is set to SCROLL or HIDDEN.

##### Box-sizing:

1. Content-box:

Not big issue, because the content height for child will be same the padding will be added to top of it.

2. Border-box:

When padding is already known. Everything is good because we can calculate the padding and substract it from parent's dimension to calculate the content-dimension.

However, if padding depends on parent's dimension, and parent dimension depends on child's (`FITCONTENT`) then, it is sure that child's will not depend on parent's because of circular dependency.Hence, in given senario, child's dimension does not depend on the parent's no matter whatever we provide in the child's **init**() . Hence we can get the parent's dimension and then calculate the padding afterwards.

#### Event-handlers:

Each event handler is FnMut(&mut EVENT), the bool value decide whether the event must be passed on to subsequent layers(bubbling or capturing) or not.

Keyboard events supports bubbling up only.

Mouse Events supports both bubbling and capture.

#### Rendering:

```
	Children's Virtual View
	______
  |       |
  |       |
  |       |
  ____________  <--- Scroll View
  |       |
  |       |
  |       |
  |       |     <--- Actual Content Window
  ____________
  |       |
  |       |
   _______
```

child will create its basic struct and parent will destroy it after copying
While rendering we will correct the child's box as per (0,0) to (height + padding, width + padding).
then while rendering we will render it from (y + border, x + border)

#### Coloring

- New Color pair will be created with current background color and text color.
- New Color pair with background color and border color for borders.

Parent must reserve the COLOR_PAIR() of its child to allow the colors to be copied to its window.
Example: If parent changed used init_pair(1) which replaces the init pair child was using this will result in different output at the screen

Hence we use a HashMap to store the pair number for all the possible pairs in the image
If pairs go above the available COLOR_PAIRS we set every further pairs to circle over from start but we don't replace the actual pair declared before.
implemented using `Document.get_color_pair()`

To keep default terminal color use negative number (every color is set to -1 by default).

- For custom colors
  - define using `Document.new_color()`
  - In normal terminal, colors are mapped in following manner..
    0 - 15 : Standard
    16 - 231 : RGB ( distributed as 6*6*6 box )
    232 - 255: Grayscale

#### Document

- Manages the global state of the app.
- Only public function of Document is `get_color`

#### Known Issues:

1. If User is passing some closure within their declared component. It may need to be passed in Arc<Mutex<>>, if the closure is FnMut().
2. Either the closure passed must implement Send + Sync or the Component must implement Send : `unsafe impl Send for T` where T : `Component`

### Next:

4. Either Fix the Send + Sync problem, or shift to single threaded
5. Event Loop
6. Extended Color support
