let Point = { x: Double, y: Double }

--- Bezier values
---
--- `a`, `b` -- any floating point value
--- `c`, `d` -- any value from [0; 1]
let Bezier = { a: Double, b: Double, c: Double, d: Double }
--- Constructor for bezier values for convenience
let bezier = \(a: Double) -> \(b: Double) -> \(c: Double) -> \(d: Double) -> { a, b, c, d }: Bezier

--- Transition animation easing function
---
--- # Example
---
--- Using bezier as easing function
---
--- ```dhall
--- let W = ./schema.dhall
--- in {
---     animation = { easing = W.Easing.Bezier (W.bezier 0.97 0.0 0.39 0.39) },
--- }
--- ```
let Easing = <
    --- Linear animation
    None |
    --- Ease in
    EaseIn |
    --- Ease out
    EaseOut |
    --- Ease in out
    EaseInOut |
    --- Bezier function
    Bezier: Bezier >

--- Circle transition animation centre position
let CircleCenter = <
    --- Means that Waywe will choose new position each time for you
    Random |
    --- You choose the exact centre position
    Point: Point >
--- Circle animation direction
let CircleDirection = <
    --- Outside in
    In |
    --- Inside out
    Out >

--- Slide transition animation angle
let SlideAngle = <
    --- Means that Waywe will choose new angle each time for you
    Random |
    --- You choose the exact slide angle
    Degrees: Double >

--- Circle transition animation
let CircleTransition = {
    --- Centre position
    center: CircleCenter,
    --- Animation direction
    direction: CircleDirection,
}
--- Slide tansition animation
let SlideTransition = {
    --- Animation angle
    angle: SlideAngle,
}

--- Transition animation
let Transition = <
    --- Circle transition animation
    Circle: CircleTransition |
    --- Slide tansition animation
    Slide: SlideTransition |
    --- Fadeout transition animation
    Fadeout >

--- Post processing effect
let Effect = <
    --- Convolutuion. Expects NxN kernel inlined in single list of values
    Convolve: List Double |
    --- Fast blur
    Blur: {
        --- Number of blur passes. More means stronger blur and lower performance.
        n_levels: Natural,
        --- Blur granularity multiplier for subsequent scaling passes.
        --- More means higher quality blur and lower performance
        level_multiplier: Natural,
    } >

--- Constructors for timelike values
let Duration = {
    --- Construct duration from milliseconds
    millis = \(count: Natural) -> count,
    --- Construct duration from seconds
    seconds = \(count: Natural) -> 1000 * count,
}

--- Transition animation config
let Animation = {
    --- Duration in milliseconds. See constructors in `Duration`
    duration: Natural,
    --- Easing for the animation
    easing: Easing,
    --- Transition style. Either circle or slide
    style: Transition,
}

--- Config file related options
let ConfigOpts = {
    --- Disabled hot reloading for config.dhall
    --- Can only be re-enabled with `waywe config reload` or `waywe restart`
    disable_hot_reload: Bool,
}

--- Waywe config values
let Config = {
    --- Transition animation config
    animation: Animation,
    --- Postprocessing effect. Will be applied in the declaration order
    effects: List Effect,
    --- Config file related options
    config: ConfigOpts,
}

let defaultConfig = {
    animation = {
        duration = Duration.seconds 2,
        easing = Easing.EaseOut,
        style = Transition.Circle {
            center = CircleCenter.Random,
            direction = CircleDirection.Out,
        }
    },
    effects = []: List Effect,
    config = {
        disable_hot_reload = False,
    },
}: Config

in {
    Point,
    Bezier,
    bezier,
    Easing,
    SlideAngle,
    CircleCenter,
    CircleDirection,
    CircleTransition,
    SlideTransition,
    Transition,
    Effect,
    Duration,
    Animation,
    Config,
    defaultConfig,
}
