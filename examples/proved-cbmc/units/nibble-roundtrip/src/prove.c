/* A `proved`-tier unit (ARCHITECTURE.md §7.2): its claim is verified over the ENTIRE
 * input space by bounded model checking, not sampled. The property mirrors the invariant
 * behind `Hash::hex` in the engine — splitting a byte into hex nibbles and recombining is
 * the identity, and every nibble maps to a valid lowercase hex digit.
 *
 * CBMC (the model checker Kani wraps) leaves `b` nondeterministic, so the assertions are
 * checked for all 256 byte values at once. Run with: `cbmc prove.c`.
 */
#include <assert.h>
#include <stdint.h>

static uint8_t hi_nibble(uint8_t b) { return b >> 4; }
static uint8_t lo_nibble(uint8_t b) { return b & 0x0f; }
static uint8_t combine(uint8_t hi, uint8_t lo) {
    return (uint8_t)((hi << 4) | (lo & 0x0f));
}

static uint8_t nibble_to_hex(uint8_t n) {
    n &= 0x0f;
    return n < 10 ? (uint8_t)('0' + n) : (uint8_t)('a' + n - 10);
}

int main(void) {
    uint8_t b; /* nondeterministic: CBMC proves the body for every possible value */

    /* 1. The split/recombine round-trip is the identity for all bytes. */
    assert(combine(hi_nibble(b), lo_nibble(b)) == b);

    /* 2. Every nibble encodes to a valid lowercase hex digit. */
    uint8_t c = nibble_to_hex(b);
    assert((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'));

    return 0;
}
