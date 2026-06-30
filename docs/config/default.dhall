let W = ./schema.dhall

in {
    --- Transition animation config
    animation = {
        --- Animation duration in milliseconds
        duration = W.Duration.seconds 2,
        --- Easing for the animation
        easing = W.Easing.EaseOut,
        --- Transition style. Either circle or slide
        style = W.Transition.Circle {
            --- Centre position
            center = W.CircleCenter.Random,
            --- Animation direction
            direction = W.CircleDirection.Out,
        }
    },
    --- Post processing effects
    effects = []: List W.Effect,
}: W.Config
