use crate::task_blink::BlinkMsg;

#[derive(Clone, Copy)]
pub enum Msg {
    Blink(BlinkMsg),
}

impl Msg {
    // What we really want is a C-style discriminant on the enum elements. But it looks like this can't be reliably
    // obtained, even using unsafe.
    //
    // - https://docs.rs/num-derive/latest/num_derive/derive.ToPrimitive.html - this doesn't work on enums with data.
    //      Probably it would be possible to write a similar macro that works on all enums.
    // - `unsafe { core::mem::transmute::<_, u32>(core::mem::discriminant(&msg)) }` - this seems to work but I can't find
    //      evidence that it's reliable. Fundamentally the enum must have some hidden tag value which is presumably
    //      the discriminant inner value, which starts from 0. But I can't guarantee it and also not completely sure
    //      that this wouldn't be UB anyway. Also: is discriminant always 32 bits on all machines?
    //
    // This function manually achieves a C-style value for each enum element.
    pub fn id(&self) -> usize {
        match self {
            Self::Blink(_) => 0,
        }
    }
}
