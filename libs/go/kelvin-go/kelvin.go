package kelvin

/*
#cgo LDFLAGS: -lkelvin_ffi
#include <stdlib.h>
#include <stdint.h>

// FFI declarations matching kelvin-ffi/src/c_api.rs
typedef struct KelvinCtx KelvinCtx;

KelvinCtx* kelvin_new(const char* config_json, char** error_out);
int32_t kelvin_encrypt(KelvinCtx* ctx, uint8_t* data, size_t len);
int32_t kelvin_decrypt(KelvinCtx* ctx, uint8_t* data, size_t len);
uint64_t kelvin_remaining_bytes(const KelvinCtx* ctx);
void kelvin_free(KelvinCtx* ctx);
*/
import "C"
import (
	"errors"
	"unsafe"
)

type Kelvin struct {
	ctx *C.KelvinCtx
}

// New creates a new Kelvin instance from a JSON configuration.
func New(configJSON string) (*Kelvin, error) {
	cConfig := C.CString(configJSON)
	defer C.free(unsafe.Pointer(cConfig))

	var cError *C.char
	ctx := C.kelvin_new(cConfig, &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.free(unsafe.Pointer(cError))
		return nil, errors.New("failed to initialize Kelvin: " + errStr)
	}

	return &Kelvin{ctx: ctx}, nil
}

// Encrypt encrypts the given data in-place.
func (k *Kelvin) Encrypt(data []byte) error {
	if len(data) == 0 {
		return nil
	}
	res := C.kelvin_encrypt(k.ctx, (*C.uint8_t)(&data[0]), C.size_t(len(data)))
	if res != 0 {
		return errors.New("encryption failed")
	}
	return nil
}

// Decrypt decrypts the given data in-place.
func (k *Kelvin) Decrypt(data []byte) error {
	// In Kelvin, encryption and decryption are the same XOR operation.
	return k.Encrypt(data)
}

// RemainingBytes returns the number of safe bytes remaining in the simulation.
func (k *Kelvin) RemainingBytes() uint64 {
	return uint64(C.kelvin_remaining_bytes(k.ctx))
}

// Close releases the resources associated with the Kelvin instance.
func (k *Kelvin) Close() {
	if k.ctx != nil {
		C.kelvin_free(k.ctx)
		k.ctx = nil
	}
}
