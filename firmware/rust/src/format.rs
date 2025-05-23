//
// Helper function to format an unsigned integer with a prefix and suffix value
//
pub fn format_uint(
    buf: &mut [u8],
    prefix: &[u8],
    value: u16,
    decimal_digits: u16,
    suffix: Option<&[u8]>,
) {
    let num_chars = buf.len();
    let prefix_len = prefix.len();

    // copy prefix to buf (i.e. "Ve:____")
    buf[..prefix_len].copy_from_slice(prefix);

    // copy suffix to buf if provided
    if let Some(suffix) = suffix {
        let suffix_len = suffix.len();
        buf[num_chars - suffix_len..].copy_from_slice(suffix);
    }

    // now copy the value by digit into buf from the right
    let mut need_decimal = decimal_digits > 0;
    let mut digits_in_buf = 0;
    let mut value = value;
    let value_len = if let Some(suffix) = suffix {
        num_chars - prefix_len - suffix.len()
    } else {
        num_chars - prefix_len
    };
    for index in (prefix_len..prefix_len + value_len).rev() {
        if need_decimal && digits_in_buf == decimal_digits {
            buf[index] = b'.';
            need_decimal = false;
        } else if value > 0 {
            buf[index] = b'0' + (value % 10) as u8;
            value /= 10;
            digits_in_buf += 1;
        } else {
            buf[index] = if digits_in_buf < (1 + decimal_digits) {
                digits_in_buf += 1;
                b'0'
            } else {
                b' '
            };
        }
    }
}

pub fn format_buf(buf: &mut [u8], left: &[u8], right: &[u8]) {
    let num_chars = buf.len();
    let left_len = left.len();
    let right_len = right.len();

    if left_len + right_len > num_chars {
        panic!("Left and right strings are too long to fit in the buffer");
    }

    buf[..left_len].copy_from_slice(left);
    buf[num_chars - right_len..].copy_from_slice(right);
    for i in left_len..num_chars - right_len {
        buf[i] = b' ';
    }
}
