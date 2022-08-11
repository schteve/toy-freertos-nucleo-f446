use freertos_rust::{FreeRtosBaseType, FreeRtosError, FreeRtosTaskHandle, FreeRtosUBaseType, Task};

pub fn task_handle(task: Option<Task>) -> FreeRtosTaskHandle {
    let task = if let Some(t) = task {
        t
    } else {
        Task::current().unwrap()
    };
    unsafe { *(&task as *const _ as *const FreeRtosTaskHandle) } // Yuck. There's no way to get at the task handle otherwise.
}

pub fn get_task_id(task: Option<Task>) -> Result<FreeRtosBaseType, FreeRtosError> {
    let task_handle = task_handle(task);
    let task_id = unsafe { freertos_rs_uxTaskGetTaskNumber(task_handle) };
    if task_id == 0 {
        Err(FreeRtosError::TaskNotFound)
    } else {
        Ok(task_id)
    }
}

pub fn set_task_id(task: Option<Task>, value: FreeRtosUBaseType) {
    let task_handle = task_handle(task);
    unsafe { freertos_rs_vTaskSetTaskNumber(task_handle, value) };
}

extern "C" {
    pub fn freertos_rs_uxTaskGetTaskNumber(task_handle: FreeRtosTaskHandle) -> FreeRtosBaseType;
    pub fn freertos_rs_vTaskSetTaskNumber(
        task_handle: FreeRtosTaskHandle,
        value: FreeRtosUBaseType,
    );
}
