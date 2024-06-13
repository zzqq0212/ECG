
const call_t syscalls[] = {
    {"pcTimerGetName", 0, {}, (syscall_t)pcTimerGetName},
    {"pvTaskGetThreadLocalStoragePointer", 0, {}, (syscall_t)pvTaskGetThreadLocalStoragePointer},
    {"pvTimerGetTimerID", 0, {}, (syscall_t)pvTimerGetTimerID},
    {"ulTaskEndTrace", 0, {}, (syscall_t)ulTaskEndTrace},
    {"ulTaskNotifyTake", 0, {}, (syscall_t)ulTaskNotifyTake},
    {"ulTaskNotifyTakeIndexed", 0, {}, (syscall_t)ulTaskNotifyTakeIndexed},
    {"uxTaskGetNumberOfTasks", 0, {}, (syscall_t)uxTaskGetNumberOfTasks},
    {"uxTaskGetSystemState", 0, {}, (syscall_t)uxTaskGetSystemState},
    {"uxTimerGetReloadMode", 0, {}, (syscall_t)uxTimerGetReloadMode},
    {"vTaskGetInfo", 0, {}, (syscall_t)vTaskGetInfo},
    {"vTaskGetRunTimeStats", 0, {}, (syscall_t)vTaskGetRunTimeStats},
    {"vTaskList", 0, {}, (syscall_t)vTaskList},
    {"vTaskNotifyGiveFromISR", 0, {}, (syscall_t)vTaskNotifyGiveFromISR},
    {"vTaskNotifyGiveIndexedFromISR", 0, {}, (syscall_t)vTaskNotifyGiveIndexedFromISR},
    {"vTaskSetApplicationTaskTag", 0, {}, (syscall_t)vTaskSetApplicationTaskTag},
    {"vTaskSetThreadLocalStoragePointer", 0, {}, (syscall_t)vTaskSetThreadLocalStoragePointer},
    {"vTaskSetTimeOutState", 0, {}, (syscall_t)vTaskSetTimeOutState},
    {"vTaskStartTrace", 0, {}, (syscall_t)vTaskStartTrace},
    {"vTimerSetReloadMode", 0, {}, (syscall_t)vTimerSetReloadMode},
    {"vTimerSetTimerID", 0, {}, (syscall_t)vTimerSetTimerID},
    {"xTaskCallApplicationTaskHook", 0, {}, (syscall_t)xTaskCallApplicationTaskHook},
    {"xTaskCheckForTimeOut", 0, {}, (syscall_t)xTaskCheckForTimeOut},
    {"xTaskGetApplicationTaskTag", 0, {}, (syscall_t)xTaskGetApplicationTaskTag},
    {"xTaskGetApplicationTaskTagFromISR", 0, {}, (syscall_t)xTaskGetApplicationTaskTagFromISR},
    {"xTaskGetCurrentTaskHandle", 0, {}, (syscall_t)xTaskGetCurrentTaskHandle},
    {"xTaskGetHandle", 0, {}, (syscall_t)xTaskGetHandle},
    {"xTaskGetIdleRunTimeCounter", 0, {}, (syscall_t)xTaskGetIdleRunTimeCounter},
    {"xTaskGetSchedulerState", 0, {}, (syscall_t)xTaskGetSchedulerState},
    {"xTaskGetTickCount", 0, {}, (syscall_t)xTaskGetTickCount},
    {"xTaskGetTickCountFromISR", 0, {}, (syscall_t)xTaskGetTickCountFromISR},
    {"xTaskNotify", 0, {}, (syscall_t)xTaskNotify},
    {"xTaskNotifyAndQuery", 0, {}, (syscall_t)xTaskNotifyAndQuery},
    {"xTaskNotifyAndQueryFromISR", 0, {}, (syscall_t)xTaskNotifyAndQueryFromISR},
    {"xTaskNotifyAndQueryIndexed", 0, {}, (syscall_t)xTaskNotifyAndQueryIndexed},
    {"xTaskNotifyAndQueryIndexedFromISR", 0, {}, (syscall_t)xTaskNotifyAndQueryIndexedFromISR},
    {"xTaskNotifyFromISR", 0, {}, (syscall_t)xTaskNotifyFromISR},
    {"xTaskNotifyGive", 0, {}, (syscall_t)xTaskNotifyGive},
    {"xTaskNotifyGiveIndexed", 0, {}, (syscall_t)xTaskNotifyGiveIndexed},
    {"xTaskNotifyIndexed", 0, {}, (syscall_t)xTaskNotifyIndexed},
    {"xTaskNotifyIndexedFromISR", 0, {}, (syscall_t)xTaskNotifyIndexedFromISR},
    {"xTaskNotifyStateClear", 0, {}, (syscall_t)xTaskNotifyStateClear},
    {"xTaskNotifyStateClearIndexed", 0, {}, (syscall_t)xTaskNotifyStateClearIndexed},
    {"xTaskNotifyWait", 0, {}, (syscall_t)xTaskNotifyWait},
    {"xTaskNotifyWaitIndexed", 0, {}, (syscall_t)xTaskNotifyWaitIndexed},
    {"xTimerChangePeriod", 0, {}, (syscall_t)xTimerChangePeriod},
    {"xTimerChangePeriodFromISR", 0, {}, (syscall_t)xTimerChangePeriodFromISR},
    {"xTimerCreate", 0, {}, (syscall_t)xTimerCreate},
    {"xTimerCreateStatic", 0, {}, (syscall_t)xTimerCreateStatic},
    {"xTimerDelete", 0, {}, (syscall_t)xTimerDelete},
    {"xTimerGetExpiryTime", 0, {}, (syscall_t)xTimerGetExpiryTime},
    {"xTimerGetPeriod", 0, {}, (syscall_t)xTimerGetPeriod},
    {"xTimerGetTimerDaemonTaskHandle", 0, {}, (syscall_t)xTimerGetTimerDaemonTaskHandle},
    {"xTimerIsTimerActive", 0, {}, (syscall_t)xTimerIsTimerActive},
    {"xTimerPendFunctionCall", 0, {}, (syscall_t)xTimerPendFunctionCall},
    {"xTimerReset", 0, {}, (syscall_t)xTimerReset},
    {"xTimerResetFromISR", 0, {}, (syscall_t)xTimerResetFromISR},
    {"xTimerStart", 0, {}, (syscall_t)xTimerStart},
    {"xTimerStartFromISR", 0, {}, (syscall_t)xTimerStartFromISR},
    {"xTimerStop", 0, {}, (syscall_t)xTimerStop},
    {"xTimerStopFromISR", 0, {}, (syscall_t)xTimerStopFromISR},
};
