/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

pub const PROJECT_NAME: &str = "Betrock.rs";
pub const PROJECT_VERSION_MAJOR: i32 = 0;
pub const PROJECT_VERSION_MINOR: i32 = 0;
pub const PROJECT_VERSION_PATCH: i32 = 1;
pub const PROJECT_VERSION_STRING: &str = "0.0.1"; // Unused until first true release
pub const PROJECT_GIT_COMMIT: &str = env!("PROJECT_GIT_COMMIT");
pub const PROJECT_GIT_BRANCH: &str = env!("PROJECT_GIT_BRANCH");
pub const PROJECT_VERSION_FULL_STRING: &str = concat!(
    "(",
    env!("PROJECT_GIT_BRANCH"),
    "/",
    env!("PROJECT_GIT_COMMIT"),
    ")"
);
pub const PROJECT_FULL_VERSION_LABEL: &str = concat!(
    "Betrock.rs (",
    env!("PROJECT_GIT_BRANCH"),
    "/",
    env!("PROJECT_GIT_COMMIT"),
    ")"
);
