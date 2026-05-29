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
void kelvin_free_string(char* s);

// Streaming encryptors
typedef struct PhotonEncryptorCtx PhotonEncryptorCtx;
PhotonEncryptorCtx* kelvin_photon_encryptor_new(const uint8_t* seed, size_t seed_len, uint64_t max_reseeds, char** error_out);
int32_t kelvin_photon_encryptor_update(PhotonEncryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_photon_encryptor_finalize(PhotonEncryptorCtx* ctx, uint8_t* tag_out, size_t tag_cap, size_t* tag_len);
void kelvin_photon_encryptor_free(PhotonEncryptorCtx* ctx);

typedef struct PhotonDecryptorCtx PhotonDecryptorCtx;
PhotonDecryptorCtx* kelvin_photon_decryptor_new(const uint8_t* seed, size_t seed_len, uint64_t max_reseeds, char** error_out);
int32_t kelvin_photon_decryptor_update(PhotonDecryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_photon_decryptor_finalize(PhotonDecryptorCtx* ctx, const uint8_t* tag, size_t tag_len);
void kelvin_photon_decryptor_free(PhotonDecryptorCtx* ctx);

typedef struct QuantumEncryptorCtx QuantumEncryptorCtx;
QuantumEncryptorCtx* kelvin_quantum_encryptor_new(const uint8_t* seed, size_t seed_len, uint64_t max_reseeds, char** error_out);
int32_t kelvin_quantum_encryptor_update(QuantumEncryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_quantum_encryptor_finalize(QuantumEncryptorCtx* ctx, uint8_t* tag_out, size_t tag_cap, size_t* tag_len);
void kelvin_quantum_encryptor_free(QuantumEncryptorCtx* ctx);

typedef struct QuantumDecryptorCtx QuantumDecryptorCtx;
QuantumDecryptorCtx* kelvin_quantum_decryptor_new(const uint8_t* seed, size_t seed_len, uint64_t max_reseeds, char** error_out);
int32_t kelvin_quantum_decryptor_update(QuantumDecryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_quantum_decryptor_finalize(QuantumDecryptorCtx* ctx, const uint8_t* tag, size_t tag_len);
void kelvin_quantum_decryptor_free(QuantumDecryptorCtx* ctx);

typedef struct ChaosEncryptorCtx ChaosEncryptorCtx;
ChaosEncryptorCtx* kelvin_chaos_encryptor_new(const char* config_json, uint64_t bytes_per_step, char** error_out);
int32_t kelvin_chaos_encryptor_update(ChaosEncryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_chaos_encryptor_finalize(ChaosEncryptorCtx* ctx, uint8_t* tag_out, size_t tag_cap, size_t* tag_len);
void kelvin_chaos_encryptor_free(ChaosEncryptorCtx* ctx);

typedef struct ChaosDecryptorCtx ChaosDecryptorCtx;
ChaosDecryptorCtx* kelvin_chaos_decryptor_new(const char* config_json, uint64_t bytes_per_step, char** error_out);
int32_t kelvin_chaos_decryptor_update(ChaosDecryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_chaos_decryptor_finalize(ChaosDecryptorCtx* ctx, const uint8_t* tag, size_t tag_len);
void kelvin_chaos_decryptor_free(ChaosDecryptorCtx* ctx);

typedef struct SecureEncryptorCtx SecureEncryptorCtx;
SecureEncryptorCtx* kelvin_secure_encryptor_new(const uint8_t* key, size_t key_len, const uint8_t* nonce, size_t nonce_len, char** error_out);
int32_t kelvin_secure_encryptor_update(SecureEncryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_secure_encryptor_finalize(SecureEncryptorCtx* ctx, uint8_t* tag_out, size_t tag_cap, size_t* tag_len);
void kelvin_secure_encryptor_free(SecureEncryptorCtx* ctx);

typedef struct SecureDecryptorCtx SecureDecryptorCtx;
SecureDecryptorCtx* kelvin_secure_decryptor_new(const uint8_t* key, size_t key_len, const uint8_t* nonce, size_t nonce_len, char** error_out);
int32_t kelvin_secure_decryptor_update(SecureDecryptorCtx* ctx, const uint8_t* input, size_t input_len, uint8_t* output, size_t output_cap, size_t* output_len);
int32_t kelvin_secure_decryptor_finalize(SecureDecryptorCtx* ctx, const uint8_t* tag, size_t tag_len);
void kelvin_secure_decryptor_free(SecureDecryptorCtx* ctx);
*/
import "C"
import (
	"errors"
	"unsafe"
)

// ErrClosed is returned when a method is called on a closed Kelvin instance.
var ErrClosed = errors.New("kelvin: instance is closed")

// ============================================================================
// V1 Kelvin — AEAD encrypt/decrypt (existing)
// ============================================================================

// Kelvin is a V1 AEAD encrypt/decrypt instance.
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
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to initialize Kelvin: " + errStr)
	}

	return &Kelvin{ctx: ctx}, nil
}

// Encrypt encrypts the given data in-place.
func (k *Kelvin) Encrypt(data []byte) error {
	if k.ctx == nil {
		return ErrClosed
	}
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
	if k.ctx == nil {
		return ErrClosed
	}
	// In Kelvin, encryption and decryption are the same XOR operation.
	return k.Encrypt(data)
}

// RemainingBytes returns the number of safe bytes remaining in the simulation.
func (k *Kelvin) RemainingBytes() uint64 {
	if k.ctx == nil {
		return 0
	}
	return uint64(C.kelvin_remaining_bytes(k.ctx))
}

// Close releases the resources associated with the Kelvin instance.
func (k *Kelvin) Close() {
	if k.ctx != nil {
		C.kelvin_free(k.ctx)
		k.ctx = nil
	}
}

// ============================================================================
// Streaming API — Generic helpers
// ============================================================================

// streamEncrypt performs a streaming encrypt update and returns the output.
func streamEncrypt(updateFn func(*C.uint8_t, C.size_t, *C.uint8_t, C.size_t, *C.size_t) C.int32_t, input []byte) ([]byte, error) {
	if len(input) == 0 {
		return []byte{}, nil
	}
	output := make([]byte, len(input))
	var outputLen C.size_t
	res := updateFn((*C.uint8_t)(&input[0]), C.size_t(len(input)), (*C.uint8_t)(&output[0]), C.size_t(len(output)), &outputLen)
	if res != 0 {
		return nil, errors.New("streaming update failed")
	}
	return output[:outputLen], nil
}

// streamFinalize performs a streaming encrypt finalize and returns the tag.
func streamFinalize(finalizeFn func(*C.uint8_t, C.size_t, *C.size_t) C.int32_t) ([]byte, error) {
	tag := make([]byte, 64) // max tag size
	var tagLen C.size_t
	res := finalizeFn((*C.uint8_t)(&tag[0]), C.size_t(len(tag)), &tagLen)
	if res != 0 {
		return nil, errors.New("streaming finalize failed")
	}
	return tag[:tagLen], nil
}

// ============================================================================
// Photon (V3) Streaming
// ============================================================================

// PhotonEncryptor is a V3 Photon streaming encryptor.
type PhotonEncryptor struct {
	ctx *C.PhotonEncryptorCtx
}

// NewPhotonEncryptor creates a new Photon streaming encryptor.
func NewPhotonEncryptor(seed []byte, maxReseeds uint64) (*PhotonEncryptor, error) {
	var cError *C.char
	ctx := C.kelvin_photon_encryptor_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create PhotonEncryptor: " + errStr)
	}
	return &PhotonEncryptor{ctx: ctx}, nil
}

// Update encrypts a chunk of plaintext.
func (e *PhotonEncryptor) Update(plaintext []byte) ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_photon_encryptor_update(e.ctx, input, inputLen, output, outputCap, outputLen)
	}, plaintext)
}

// Finalize finalizes encryption and returns the authentication tag (empty for Photon).
func (e *PhotonEncryptor) Finalize() ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamFinalize(func(tagOut *C.uint8_t, tagCap C.size_t, tagLen *C.size_t) C.int32_t {
		return C.kelvin_photon_encryptor_finalize(e.ctx, tagOut, tagCap, tagLen)
	})
}

// Close releases resources.
func (e *PhotonEncryptor) Close() {
	if e.ctx != nil {
		C.kelvin_photon_encryptor_free(e.ctx)
		e.ctx = nil
	}
}

// PhotonDecryptor is a V3 Photon streaming decryptor.
type PhotonDecryptor struct {
	ctx *C.PhotonDecryptorCtx
}

// NewPhotonDecryptor creates a new Photon streaming decryptor.
func NewPhotonDecryptor(seed []byte, maxReseeds uint64) (*PhotonDecryptor, error) {
	var cError *C.char
	ctx := C.kelvin_photon_decryptor_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create PhotonDecryptor: " + errStr)
	}
	return &PhotonDecryptor{ctx: ctx}, nil
}

// Update decrypts a chunk of ciphertext.
func (d *PhotonDecryptor) Update(ciphertext []byte) ([]byte, error) {
	if d.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_photon_decryptor_update(d.ctx, input, inputLen, output, outputCap, outputLen)
	}, ciphertext)
}

// Finalize verifies the authentication tag (no-op for Photon).
func (d *PhotonDecryptor) Finalize(tag []byte) error {
	if d.ctx == nil {
		return ErrClosed
	}
	var tagPtr *C.uint8_t
	var tagLen C.size_t
	if len(tag) > 0 {
		tagPtr = (*C.uint8_t)(&tag[0])
		tagLen = C.size_t(len(tag))
	}
	res := C.kelvin_photon_decryptor_finalize(d.ctx, tagPtr, tagLen)
	if res != 0 {
		return errors.New("streaming finalize failed")
	}
	return nil
}

// Close releases resources.
func (d *PhotonDecryptor) Close() {
	if d.ctx != nil {
		C.kelvin_photon_decryptor_free(d.ctx)
		d.ctx = nil
	}
}

// ============================================================================
// Quantum (H) Streaming
// ============================================================================

// QuantumEncryptor is an H Quantum streaming encryptor.
type QuantumEncryptor struct {
	ctx *C.QuantumEncryptorCtx
}

// NewQuantumEncryptor creates a new Quantum streaming encryptor.
func NewQuantumEncryptor(seed []byte, maxReseeds uint64) (*QuantumEncryptor, error) {
	var cError *C.char
	ctx := C.kelvin_quantum_encryptor_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create QuantumEncryptor: " + errStr)
	}
	return &QuantumEncryptor{ctx: ctx}, nil
}

// Update encrypts a chunk of plaintext.
func (e *QuantumEncryptor) Update(plaintext []byte) ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_quantum_encryptor_update(e.ctx, input, inputLen, output, outputCap, outputLen)
	}, plaintext)
}

// Finalize finalizes encryption (no-op for Quantum).
func (e *QuantumEncryptor) Finalize() ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamFinalize(func(tagOut *C.uint8_t, tagCap C.size_t, tagLen *C.size_t) C.int32_t {
		return C.kelvin_quantum_encryptor_finalize(e.ctx, tagOut, tagCap, tagLen)
	})
}

// Close releases resources.
func (e *QuantumEncryptor) Close() {
	if e.ctx != nil {
		C.kelvin_quantum_encryptor_free(e.ctx)
		e.ctx = nil
	}
}

// QuantumDecryptor is an H Quantum streaming decryptor.
type QuantumDecryptor struct {
	ctx *C.QuantumDecryptorCtx
}

// NewQuantumDecryptor creates a new Quantum streaming decryptor.
func NewQuantumDecryptor(seed []byte, maxReseeds uint64) (*QuantumDecryptor, error) {
	var cError *C.char
	ctx := C.kelvin_quantum_decryptor_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create QuantumDecryptor: " + errStr)
	}
	return &QuantumDecryptor{ctx: ctx}, nil
}

// Update decrypts a chunk of ciphertext.
func (d *QuantumDecryptor) Update(ciphertext []byte) ([]byte, error) {
	if d.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_quantum_decryptor_update(d.ctx, input, inputLen, output, outputCap, outputLen)
	}, ciphertext)
}

// Finalize verifies the authentication tag (no-op for Quantum).
func (d *QuantumDecryptor) Finalize(tag []byte) error {
	if d.ctx == nil {
		return ErrClosed
	}
	var tagPtr *C.uint8_t
	var tagLen C.size_t
	if len(tag) > 0 {
		tagPtr = (*C.uint8_t)(&tag[0])
		tagLen = C.size_t(len(tag))
	}
	res := C.kelvin_quantum_decryptor_finalize(d.ctx, tagPtr, tagLen)
	if res != 0 {
		return errors.New("streaming finalize failed")
	}
	return nil
}

// Close releases resources.
func (d *QuantumDecryptor) Close() {
	if d.ctx != nil {
		C.kelvin_quantum_decryptor_free(d.ctx)
		d.ctx = nil
	}
}

// ============================================================================
// Chaos (V2) Streaming
// ============================================================================

// ChaosEncryptor is a V2 Chaos streaming encryptor.
type ChaosEncryptor struct {
	ctx *C.ChaosEncryptorCtx
}

// NewChaosEncryptor creates a new Chaos streaming encryptor from a JSON config.
func NewChaosEncryptor(configJSON string, bytesPerStep uint64) (*ChaosEncryptor, error) {
	cConfig := C.CString(configJSON)
	defer C.free(unsafe.Pointer(cConfig))

	var cError *C.char
	ctx := C.kelvin_chaos_encryptor_new(cConfig, C.uint64_t(bytesPerStep), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create ChaosEncryptor: " + errStr)
	}
	return &ChaosEncryptor{ctx: ctx}, nil
}

// Update encrypts a chunk of plaintext.
func (e *ChaosEncryptor) Update(plaintext []byte) ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_chaos_encryptor_update(e.ctx, input, inputLen, output, outputCap, outputLen)
	}, plaintext)
}

// Finalize finalizes encryption (no-op for Chaos).
func (e *ChaosEncryptor) Finalize() ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamFinalize(func(tagOut *C.uint8_t, tagCap C.size_t, tagLen *C.size_t) C.int32_t {
		return C.kelvin_chaos_encryptor_finalize(e.ctx, tagOut, tagCap, tagLen)
	})
}

// Close releases resources.
func (e *ChaosEncryptor) Close() {
	if e.ctx != nil {
		C.kelvin_chaos_encryptor_free(e.ctx)
		e.ctx = nil
	}
}

// ChaosDecryptor is a V2 Chaos streaming decryptor.
type ChaosDecryptor struct {
	ctx *C.ChaosDecryptorCtx
}

// NewChaosDecryptor creates a new Chaos streaming decryptor from a JSON config.
func NewChaosDecryptor(configJSON string, bytesPerStep uint64) (*ChaosDecryptor, error) {
	cConfig := C.CString(configJSON)
	defer C.free(unsafe.Pointer(cConfig))

	var cError *C.char
	ctx := C.kelvin_chaos_decryptor_new(cConfig, C.uint64_t(bytesPerStep), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create ChaosDecryptor: " + errStr)
	}
	return &ChaosDecryptor{ctx: ctx}, nil
}

// Update decrypts a chunk of ciphertext.
func (d *ChaosDecryptor) Update(ciphertext []byte) ([]byte, error) {
	if d.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_chaos_decryptor_update(d.ctx, input, inputLen, output, outputCap, outputLen)
	}, ciphertext)
}

// Finalize verifies the authentication tag (no-op for Chaos).
func (d *ChaosDecryptor) Finalize(tag []byte) error {
	if d.ctx == nil {
		return ErrClosed
	}
	var tagPtr *C.uint8_t
	var tagLen C.size_t
	if len(tag) > 0 {
		tagPtr = (*C.uint8_t)(&tag[0])
		tagLen = C.size_t(len(tag))
	}
	res := C.kelvin_chaos_decryptor_finalize(d.ctx, tagPtr, tagLen)
	if res != 0 {
		return errors.New("streaming finalize failed")
	}
	return nil
}

// Close releases resources.
func (d *ChaosDecryptor) Close() {
	if d.ctx != nil {
		C.kelvin_chaos_decryptor_free(d.ctx)
		d.ctx = nil
	}
}

// ============================================================================
// Secure (V1) Streaming
// ============================================================================

// SecureEncryptor is a V1 Secure streaming encryptor (ChaCha20 + BLAKE3).
type SecureEncryptor struct {
	ctx *C.SecureEncryptorCtx
}

// NewSecureEncryptor creates a new Secure streaming encryptor.
func NewSecureEncryptor(key [32]byte, nonce [12]byte) (*SecureEncryptor, error) {
	var cError *C.char
	ctx := C.kelvin_secure_encryptor_new((*C.uint8_t)(&key[0]), 32, (*C.uint8_t)(&nonce[0]), 12, &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create SecureEncryptor: " + errStr)
	}
	return &SecureEncryptor{ctx: ctx}, nil
}

// Update encrypts a chunk of plaintext.
func (e *SecureEncryptor) Update(plaintext []byte) ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_secure_encryptor_update(e.ctx, input, inputLen, output, outputCap, outputLen)
	}, plaintext)
}

// Finalize finalizes encryption and returns the BLAKE3 authentication tag.
func (e *SecureEncryptor) Finalize() ([]byte, error) {
	if e.ctx == nil {
		return nil, ErrClosed
	}
	return streamFinalize(func(tagOut *C.uint8_t, tagCap C.size_t, tagLen *C.size_t) C.int32_t {
		return C.kelvin_secure_encryptor_finalize(e.ctx, tagOut, tagCap, tagLen)
	})
}

// Close releases resources.
func (e *SecureEncryptor) Close() {
	if e.ctx != nil {
		C.kelvin_secure_encryptor_free(e.ctx)
		e.ctx = nil
	}
}

// SecureDecryptor is a V1 Secure streaming decryptor.
type SecureDecryptor struct {
	ctx *C.SecureDecryptorCtx
}

// NewSecureDecryptor creates a new Secure streaming decryptor.
func NewSecureDecryptor(key [32]byte, nonce [12]byte) (*SecureDecryptor, error) {
	var cError *C.char
	ctx := C.kelvin_secure_decryptor_new((*C.uint8_t)(&key[0]), 32, (*C.uint8_t)(&nonce[0]), 12, &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create SecureDecryptor: " + errStr)
	}
	return &SecureDecryptor{ctx: ctx}, nil
}

// Update decrypts a chunk of ciphertext.
func (d *SecureDecryptor) Update(ciphertext []byte) ([]byte, error) {
	if d.ctx == nil {
		return nil, ErrClosed
	}
	return streamEncrypt(func(input *C.uint8_t, inputLen C.size_t, output *C.uint8_t, outputCap C.size_t, outputLen *C.size_t) C.int32_t {
		return C.kelvin_secure_decryptor_update(d.ctx, input, inputLen, output, outputCap, outputLen)
	}, ciphertext)
}

// Finalize verifies the BLAKE3 authentication tag.
func (d *SecureDecryptor) Finalize(tag []byte) error {
	if d.ctx == nil {
		return ErrClosed
	}
	var tagPtr *C.uint8_t
	var tagLen C.size_t
	if len(tag) > 0 {
		tagPtr = (*C.uint8_t)(&tag[0])
		tagLen = C.size_t(len(tag))
	}
	res := C.kelvin_secure_decryptor_finalize(d.ctx, tagPtr, tagLen)
	if res != 0 {
		return errors.New("streaming finalize failed (tag mismatch)")
	}
	return nil
}

// Close releases resources.
func (d *SecureDecryptor) Close() {
	if d.ctx != nil {
		C.kelvin_secure_decryptor_free(d.ctx)
		d.ctx = nil
	}
}

// ============================================================================
// Prism — OTP Key Generator for Homomorphic Encryption
// ============================================================================

// Prism is an OTP key generator for homomorphic encryption.
type Prism struct {
	ctx *C.PrismCtx
}

// NewPrism creates a new Prism instance from a 2048-byte seed.
func NewPrism(seed []byte, maxReseeds uint64) (*Prism, error) {
	if len(seed) != 2048 {
		return nil, errors.New("seed must be exactly 2048 bytes")
	}
	var cError *C.char
	ctx := C.kelvin_prism_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create Prism: " + errStr)
	}
	return &Prism{ctx: ctx}, nil
}

// GenerateOTPKey generates an OTP key of the given length.
func (p *Prism) GenerateOTPKey(length int) ([]byte, error) {
	if p.ctx == nil {
		return nil, ErrClosed
	}
	if length <= 0 {
		return nil, errors.New("length must be greater than 0")
	}
	output := make([]byte, length)
	res := C.kelvin_prism_generate_otp_key(p.ctx, (*C.uint8_t)(&output[0]), C.size_t(length))
	if res != 0 {
		return nil, errors.New("generate_otp_key failed")
	}
	return output, nil
}

// SplitKey splits a key into two pads (A, B) where A xor B = K.
func (p *Prism) SplitKey(length int) ([]byte, []byte, error) {
	if p.ctx == nil {
		return nil, nil, ErrClosed
	}
	a := make([]byte, length)
	b := make([]byte, length)
	res := C.kelvin_prism_split_key(p.ctx, C.size_t(length), (*C.uint8_t)(&a[0]), (*C.uint8_t)(&b[0]))
	if res != 0 {
		return nil, nil, errors.New("split_key failed")
	}
	return a, b, nil
}

// Encrypt encrypts data in-place using Prism's domain-separated keystream.
func (p *Prism) Encrypt(data []byte) error {
	if p.ctx == nil {
		return ErrClosed
	}
	if len(data) == 0 {
		return nil
	}
	res := C.kelvin_prism_encrypt(p.ctx, (*C.uint8_t)(&data[0]), C.size_t(len(data)))
	if res != 0 {
		return errors.New("prism encrypt failed")
	}
	return nil
}

// Decrypt decrypts data in-place using Prism's domain-separated keystream.
func (p *Prism) Decrypt(data []byte) error { return p.Encrypt(data) }

// Close releases resources.
func (p *Prism) Close() {
	if p.ctx != nil {
		C.kelvin_prism_free(p.ctx)
		p.ctx = nil
	}
}

// ============================================================================
// Split — XOR Key-Splitter for Homomorphic Encryption
// ============================================================================

// Split is an XOR key-splitter for homomorphic encryption.
type Split struct {
	ctx *C.SplitCtx
}

// NewSplit creates a new Split instance from a 2048-byte seed.
func NewSplit(seed []byte, maxReseeds uint64) (*Split, error) {
	var cError *C.char
	ctx := C.kelvin_split_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create Split: " + errStr)
	}
	return &Split{ctx: ctx}, nil
}

// GenerateMasterKey generates a master key of the given length.
func (s *Split) GenerateMasterKey(length int) ([]byte, error) {
	if s.ctx == nil {
		return nil, ErrClosed
	}
	output := make([]byte, length)
	res := C.kelvin_split_generate_master_key(s.ctx, (*C.uint8_t)(&output[0]), C.size_t(length))
	if res != 0 {
		return nil, errors.New("generate_master_key failed")
	}
	return output, nil
}

// SplitKey splits a master key into two pads (A, B) where A xor B = K.
func (s *Split) SplitKey(length int) ([]byte, []byte, error) {
	if s.ctx == nil {
		return nil, nil, ErrClosed
	}
	a := make([]byte, length)
	b := make([]byte, length)
	res := C.kelvin_split_key(s.ctx, C.size_t(length), (*C.uint8_t)(&a[0]), (*C.uint8_t)(&b[0]))
	if res != 0 {
		return nil, nil, errors.New("split_key failed")
	}
	return a, b, nil
}

// Close releases resources.
func (s *Split) Close() {
	if s.ctx != nil {
		C.kelvin_split_free(s.ctx)
		s.ctx = nil
	}
}

// ============================================================================
// Flare — Chaotic FHE Secret Key Generator
// ============================================================================

// FlareScheme constants
const (
	FlareSchemeBFV  = 0
	FlareSchemeCKKS = 1
	FlareSchemeTFHE = 2
)

// Flare is a chaotic FHE secret key generator.
type Flare struct {
	ctx *C.FlareCtx
}

// NewFlare creates a new Flare instance from a 2048-byte seed.
func NewFlare(seed []byte, maxReseeds uint64) (*Flare, error) {
	var cError *C.char
	ctx := C.kelvin_flare_new((*C.uint8_t)(&seed[0]), C.size_t(len(seed)), C.uint64_t(maxReseeds), &cError)
	if ctx == nil {
		errStr := C.GoString(cError)
		C.kelvin_free_string(cError)
		return nil, errors.New("failed to create Flare: " + errStr)
	}
	return &Flare{ctx: ctx}, nil
}

// GenerateSecretKey generates a raw FHE secret key.
func (f *Flare) GenerateSecretKey(length int) ([]byte, error) {
	if f.ctx == nil {
		return nil, ErrClosed
	}
	output := make([]byte, length)
	res := C.kelvin_flare_generate_secret_key(f.ctx, (*C.uint8_t)(&output[0]), C.size_t(length))
	if res != 0 {
		return nil, errors.New("generate_secret_key failed")
	}
	return output, nil
}

// GenerateFHEKey generates an FHE secret key for the specified scheme.
func (f *Flare) GenerateFHEKey(scheme int32, length int) ([]byte, error) {
	if f.ctx == nil {
		return nil, ErrClosed
	}
	output := make([]byte, length)
	res := C.kelvin_flare_generate_fhe_key(f.ctx, C.int32_t(scheme), (*C.uint8_t)(&output[0]), C.size_t(length))
	if res != 0 {
		return nil, errors.New("generate_fhe_key failed")
	}
	return output, nil
}

// Close releases resources.
func (f *Flare) Close() {
	if f.ctx != nil {
		C.kelvin_flare_free(f.ctx)
		f.ctx = nil
	}
}
