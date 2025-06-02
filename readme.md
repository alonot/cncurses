# CNCURSES

* Can create a component.
* Custom setstate structs can be made by implementing Stateful trait

## Rendering:

The IView tree contains the basic ncurses WINDOW to render the contents. If there is any change in IView tree (style.render = True) then first __init__ is called on the IView tree root, which will in-turn call __init__ on its child. Child returns (lines, cols) which is the lines and cols it took(may be given by the user or calculate by its own child).

The actual rendering is done by __render__ method. It returns (topleftx, toplefty, lines, col) depicting the dirty rectangle, which has been rendered by itself or by its children. It is clear that on first call to __render__ this area will eventually be the area of whole screen. Parent copies the window of child into itself using two information. 1. bounds given by children, 2. its own scroll position. 

* This copying may be more optimized by returning a list of non-overlapping bounds. However, the cost of multiple call to copywin() has to be taken to account then.