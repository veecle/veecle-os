# FreeRTOS on STM32

This document explains some nuances with FreeRTOS on STM32 that the examples implement.

## Timer

The [FreeRTOS timer][FreeRTOS-Timer] is the task in charge of managing all time-based features, on which Veecle OS relies.
This timer has to be explicitly enabled in the [FreeRTOSConfig.h](../FreeRTOSConfig.h) file through the `configUSE_TIMERS` constant.
Any value other than `0` (which stands for `False`), will be treated as `True`; and then FreeRTOS will schedule a special task targeting timer features.

This "special" task has its own stack, whose size is defined by the `configTIMER_TASK_STACK_DEPTH` constant.
If your code exceeds the stack size, then the timer crashes without much diagnostic information.
These crashes can vary depending on small code changes, so if you experience unexplained crashes, review this setting.

`configTIMER_TASK_PRIORITY` is an important value.
If any task with a higher priority busy loops, then the timer does not fire.
Refer to [the FreeRTOS timer documentation for this value][FreeRTOS-Timer] for further information.

Some documentation recommends defining `configTIMER_TASK_PRIORITY` as `configMAX_PRIORITIES - 1`.
This should not be necessary.
For more information about how priorities work, please address to the [official page][FreeRTOS-priorities]

[FreeRTOS-Timer]: https://www.freertos.org/Documentation/02-Kernel/02-Kernel-features/05-Software-timers/03-Timer-daemon-configuration

[FreeRTOS-priorities]: https://www.freertos.org/Documentation/02-Kernel/02-Kernel-features/01-Tasks-and-co-routines/03-Task-priorities
