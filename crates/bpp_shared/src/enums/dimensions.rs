/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// Used as the identifier for the current dimension
// The End would become Dimenion id 1

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Dimension {
    // The nether, also known as hell or the slip
    Nether = -1,
    // The overworld, default dimension
    Overworld = 0,
}
