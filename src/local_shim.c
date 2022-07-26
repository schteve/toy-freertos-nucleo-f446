/*
    FreeRTOS shim extension - stuff that's not handled by freertos-cargo-build
*/

#include "FreeRTOS.h"
#include "task.h"

#if (configUSE_TRACE_FACILITY == 1)
BaseType_t freertos_rs_uxTaskGetTaskNumber(TaskHandle_t task) {
    return uxTaskGetTaskNumber(task);
}

void freertos_rs_vTaskSetTaskNumber(TaskHandle_t task, const UBaseType_t value) {
    return vTaskSetTaskNumber(task, value);
}
#endif
