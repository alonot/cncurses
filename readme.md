# CNCURSES

* Can create a component.
* Custom setstate structs can be made by implementing Stateful trait

## Rendering:

The IView tree contains the basic ncurses WINDOW to render the contents. If there is any change in IView tree (style.render = True) then first __init__ is called on the IView tree root, which will in-turn call __init__ on its child. Child returns (lines, cols) which is the lines and cols it took(may be given by the user or calculate by its own child).

The actual rendering is done by __render__ method. It returns (topleftx, toplefty, lines, col) depicting the dirty rectangle, which has been rendered by itself or by its children. It is clear that on first call to __render__ this area will eventually be the area of whole screen. Parent copies the window of child into itself using two information. 1. bounds given by children, 2. its own scroll position. 

* This copying may be more optimized by returning a list of non-overlapping bounds. However, the cost of multiple call to copywin() has to be taken to account then.

#### Internals:
* All texts will use **pad**
* In __init__(), if child has flex attributes then parent will set the dimensions as percentage for the children
* Flex direction, and top , bottom, left ,right all these play effect in __render__() function. 

NOTE: Dimension -> Height/Width

##### Visibility:

* Each IView will generate a window according to its `content` Dimension, not its actual Dimension. In case of OVERFLOW set to VISIBLE, the `content` Dimension will be equal to include all its child's independent of the actual Dimension. The parent while rendering will use the actual Dimensions to decide where to place the child. However the actual render will be with help of child's `content` Dimension.

##### Placing Child:

* Parent will keep `top`, `left` pointer to know which child should be rendered. The `cursorx` and `cursory` will specify to the actual location on parent's screen. Child render box will be clipped according to the scroll view, if visibility is set to SCROLL or HIDDEN. 

##### Box-sizing:

1. Content-box:
    Not big issue, because the content height for child will be same the padding will be added to top of it.
2. Border-box:
    When padding is already known. Everything is good because we can calculate the padding and substract it from parent's dimension to calculate the content-dimension.
    However, if padding depends on parent's dimension, and parent dimension depends on child's (`FITCONTENT`) then, it is sure that child's will not depend on parent's because of circular dependency.Hence, in given senario, child's dimension does not depend on the parent's no matter whatever we provide in the child's __init__() . Hence we can get the parent's dimension and then calculate the padding afterwards.

#### Event-handlers:
Each event handler is FnMut(&mut EVENT), the bool value decide whether the event must be passed on to subsequent layers(bubbling or capturing) or not.

Keyboard events supports bubbling up only. 
Mouse Events supports both bubbling and capture.

### Next:

0. Focus
2. Scroll and onscroll
3. Improving the user interface