// Copyright 2020 DCS Corporation, All Rights Reserved.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// DISTRIBUTION A. Approved for public release; distribution unlimited.
// OPSEC #4584.

use crate::rcl_bindings::*;
use crate::SubscriptionBase;

use alloc::sync::Weak;
use core::borrow::BorrowMut;
use core::fmt::Display;
use core_error::Error;
use rclrs_common::error::{to_rcl_result, RclReturnCode, WaitSetErrorCode};

#[derive(Debug)]
pub enum WaitSetErrorResponse {
    DroppedSubscription,
    ReturnCode(RclReturnCode),
}

impl Display for WaitSetErrorResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DroppedSubscription => {
                write!(f, "WaitSet: Attempted to access dropped subscription!")
            }
            Self::ReturnCode(code) => write!(f, "WaitSet: Operation returned Rcl error - {}", code),
        }
    }
}

impl From<RclReturnCode> for WaitSetErrorResponse {
    fn from(code: RclReturnCode) -> Self {
        Self::ReturnCode(code)
    }
}

impl Error for WaitSetErrorResponse {}

pub struct WaitSet {
    pub wait_set: rcl_wait_set_t,
    initialized: bool,
}

impl WaitSet {
    /// Creates and initializes a new WaitSet object.
    ///
    /// Under the hood, this calls `rcl_get_zero_initialized_wait_set()`, and stores it
    /// within the WaitSet struct, while also noting that the returned value is uninitialized.
    pub fn new(
        number_of_subscriptions: usize,
        number_of_guard_conditions: usize,
        number_of_timers: usize,
        number_of_clients: usize,
        number_of_services: usize,
        number_of_events: usize,
        context: &mut rcl_context_t,
    ) -> Result<Self, WaitSetErrorResponse> {
        let mut waitset = Self {
            wait_set: unsafe { rcl_get_zero_initialized_wait_set() },
            initialized: false,
        };
        unsafe {
            match to_rcl_result(rcl_wait_set_init(
                waitset.wait_set.borrow_mut() as *mut _,
                number_of_subscriptions,
                number_of_guard_conditions,
                number_of_timers,
                number_of_clients,
                number_of_services,
                number_of_events,
                context,
                rcutils_get_default_allocator(),
            )) {
                Ok(()) => {
                    waitset.initialized = true;
                    Ok(waitset)
                }
                Err(err) => {
                    waitset.initialized = false;
                    Err(WaitSetErrorResponse::ReturnCode(err))
                }
            }
        }
    }

    /// Removes (sets to NULL) all entities in the WaitSet
    ///
    /// # Errors
    /// - `RclError::InvalidArgument` if any arguments are invalid.
    /// - `RclError::WaitSetInvalid` if the WaitSet is already zero-initialized.
    /// - `RclError::Error` for an unspecified error
    pub fn clear(&mut self) -> Result<(), WaitSetErrorResponse> {
        if !self.initialized {
            return Err(WaitSetErrorResponse::ReturnCode(
                RclReturnCode::WaitSetError(WaitSetErrorCode::WaitSetInvalid),
            ));
        }
        unsafe {
            // Whether or not we successfully clear, this WaitSet will count as uninitialized
            self.initialized = false;
            to_rcl_result(rcl_wait_set_clear(self.wait_set.borrow_mut() as *mut _))
                .map_err(WaitSetErrorResponse::ReturnCode)
        }
    }

    /// Adds a subscription to the WaitSet
    ///
    /// # Errors
    /// - `WaitSetError::DroppedSubscription` if the passed weak pointer refers to a dropped subscription
    /// - `WaitSetError::RclError` for any `rcl` errors that occur during the process
    pub fn add_subscription(
        &mut self,
        subscription: &Weak<dyn SubscriptionBase>,
    ) -> Result<(), WaitSetErrorResponse> {
        if let Some(subscription) = subscription.upgrade() {
            let subscription_handle = &mut *subscription.handle().lock();
            unsafe {
                return to_rcl_result(rcl_wait_set_add_subscription(
                    self.wait_set.borrow_mut() as *mut _,
                    subscription_handle as *const _,
                    core::ptr::null_mut(),
                ))
                .map_err(WaitSetErrorResponse::ReturnCode);
            }
        } else {
            Err(WaitSetErrorResponse::DroppedSubscription)
        }
    }

    /// Blocks until the WaitSet is ready, or until the timeout has been exceeded
    ///
    /// This function will collect the items in the rcl_wait_set_t and pass them
    /// to the underlying rmw_wait function.
    /// The items in the wait set will be either left untouched or set to NULL after
    /// this function returns.
    /// Items that are not NULL are ready, where ready means different things based
    /// on the type of the item.
    /// For subscriptions this means there may be messages that can be taken, or
    /// perhaps that the state of the subscriptions has changed, in which case
    /// rcl_take may succeed but return with taken == false.
    /// For guard conditions this means the guard condition was triggered.
    ///
    /// The wait set struct must be allocated, initialized, and should have been
    /// cleared and then filled with items, e.g. subscriptions and guard conditions.
    /// Passing a wait set with no wait-able items in it will fail.
    /// NULL items in the sets are ignored, e.g. it is valid to have as input:
    /// subscriptions[0] = valid pointer
    /// subscriptions[1] = NULL
    /// subscriptions[2] = valid pointer
    /// size_of_subscriptions = 3
    ///
    /// Passing an uninitialized (zero initialized) wait set struct will fail.
    /// Passing a wait set struct with uninitialized memory is undefined behavior.
    /// For this reason, it is advised to use the WaitSet struct to call `wait`, as it
    /// cannot be created uninitialized.
    ///
    /// The unit of timeout is nanoseconds.
    /// If the timeout is negative then this function will block indefinitely until
    /// something in the wait set is valid or it is interrupted.
    /// If the timeout is 0 then this function will be non-blocking; checking what's
    /// ready now, but not waiting if nothing is ready yet.
    /// If the timeout is greater than 0 then this function will return after
    /// that period of time has elapsed or the wait set becomes ready, which ever
    /// comes first.
    /// Passing a timeout struct with uninitialized memory is undefined behavior.
    ///
    /// This function is thread-safe for unique wait sets with unique contents.
    /// This function cannot operate on the same wait set in multiple threads, and
    /// the wait sets may not share content.
    /// For example, calling wait() in two threads on two different wait sets
    /// that both contain a single, shared guard condition is undefined behavior.
    /// # Errors
    /// - `RclError::InvalidArgument` if an argument was invalid
    /// - `RclError::WaitSetInvalid` if the wait set is zero initialized
    /// - `RclError::WaitSetEmpty` if the wait set contains no items
    /// - `RclError::Timeout` if the timeout expired before something was ready
    /// - `RclError::Error` for an unspecified error
    pub fn wait(&mut self, timeout: i64) -> Result<(), RclReturnCode> {
        unsafe { to_rcl_result(rcl_wait(self.wait_set.borrow_mut() as *mut _, timeout)) }
    }
}

impl Drop for WaitSet {
    /// Drops the WaitSet, and clears the memory
    ///
    /// # Panics
    /// A panic is raised if `rcl` is unable to release the waitset for any reason.
    fn drop(&mut self) {
        let handle = &mut *self.wait_set.borrow_mut();
        unsafe {
            match to_rcl_result(rcl_wait_set_fini(handle as *mut _)) {
                Ok(()) => (),
                Err(err) => {
                    panic!("Unable to release WaitSet!! {:?}", err)
                }
            }
        }
    }
}
