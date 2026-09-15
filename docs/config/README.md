# Configuration

Waywe reads a Dhall config file at startup. If no valid file is found, built-in defaults are used.

Waywe does not create a config file for you. On Unix, it looks in this order:

1. `$XDG_CONFIG_HOME/waywe/config.dhall`
2. `$HOME/.config/waywe/config.dhall`
3. `/etc/waywe/config.dhall`

Copy [`schema.dhall`](schema.dhall) next to your config (or point the import at a copy elsewhere) and start from [`default.dhall`](default.dhall).

## Quick start

```dhall
let W = ./schema.dhall

in {
    animation = {
        duration = W.Duration.seconds 2,
        easing = W.Easing.EaseOut,
        style = W.Transition.Circle {
            center = W.CircleCenter.Random,
            direction = W.CircleDirection.Out,
        },
    },
    effects = []: List W.Effect,
}: W.Config
```

Put that in `~/.config/waywe/config.dhall` together with `schema.dhall`.

## Animation

| Field | Type | Description |
| --- | --- | --- |
| `duration` | `Natural` | Transition length in milliseconds. Use `W.Duration.millis` or `W.Duration.seconds`. |
| `easing` | `W.Easing` | Interpolation curve for the transition. |
| `style` | `W.Transition` | Circle or slide transition. |

### Easing

| Value | Curve |
| --- | --- |
| `W.Easing.None` | Linear: `f(t) = t` |
| `W.Easing.EaseIn` | `f(t) = t²` |
| `W.Easing.EaseOut` | `f(t) = 1 - (1 - t)²` |
| `W.Easing.EaseInOut` | `f(t) = 3t² - 2t³` |
| `W.Easing.Bezier (W.bezier a b c d)` | Cubic Bézier; `a` and `c` are clamped to `[0, 1]` |

Example with a custom curve:

```dhall
let W = ./schema.dhall

in {
    animation = {
        duration = W.Duration.seconds 2,
        easing = W.Easing.Bezier (W.bezier 0.97 0.0 0.39 0.39),
        style = W.Transition.Circle {
            center = W.CircleCenter.Random,
            direction = W.CircleDirection.Out,
        },
    },
    effects = []: List W.Effect,
}: W.Config
```

### Circle transition

```dhall
W.Transition.Circle {
    center = W.CircleCenter.Random,
    direction = W.CircleDirection.Out,
}
```

- `center` — `W.CircleCenter.Random` picks a new centre each transition. `W.CircleCenter.Point { x, y }` fixes it. Coordinates are in `[-1.0, 1.0]` on both axes, with `(0, 0)` at the screen centre.
- `direction` — `W.CircleDirection.In` reveals from the outside in; `Out` hides from the inside out.

Fixed centre:

```dhall
center = W.CircleCenter.Point { x = 0.0, y = -0.5 }
```

### Slide transition

```dhall
W.Transition.Slide {
    angle = W.SlideAngle.Random,
}
```

- `angle` — `W.SlideAngle.Random` picks a new angle (0–360°) each transition. `W.SlideAngle.Degrees 90` uses a fixed angle.

## Post-processing effects

Effects run in list order after the transition animation.

### Convolution

Apply an N×N kernel flattened into a list (row by row):

```dhall
effects = [
    W.Effect.Convolve [
        0.0, -1.0, 0.0,
        -1.0, 5.0, -1.0,
        0.0, -1.0, 0.0,
    ],
]
```

The example above is a simple sharpen filter.

### Blur

```dhall
effects = [
    W.Effect.Blur {
        n_levels = 4,
        level_multiplier = 2,
    },
]
```

- `n_levels` — number of blur passes; higher values mean stronger blur and more GPU work.
- `level_multiplier` — scales blur radius on each downsample step; higher values improve quality at the cost of performance.

## Full example

Circle transition with easing, sharpen, and blur:

```dhall
let W = ./schema.dhall

in {
    animation = {
        duration = W.Duration.millis 1500,
        easing = W.Easing.EaseInOut,
        style = W.Transition.Circle {
            center = W.CircleCenter.Point { x = 0.0, y = 0.0 },
            direction = W.CircleDirection.In,
        },
    },
    effects = [
        W.Effect.Convolve [
            0.0, -1.0, 0.0,
            -1.0, 5.0, -1.0,
            0.0, -1.0, 0.0,
        ],
        W.Effect.Blur {
            n_levels = 4,
            level_multiplier = 2,
        },
    ],
} : W.Config
```

## Schema reference

[`schema.dhall`](schema.dhall) defines all types and constructors (`W.Easing`, `W.Transition`, `W.Effect`, and so on). [`default.dhall`](default.dhall) is the same as the built-in defaults: a 2-second ease-out circle wipe from a random centre, with no effects.

If Dhall is installed, you can type-check a config before restarting Waywe:

```shell
dhall normalize ~/.config/waywe/config.dhall
```
