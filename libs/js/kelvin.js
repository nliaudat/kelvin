/**
 * Kelvin Cryptosystem — JavaScript/Node.js bindings via C FFI (ffi-napi).
 *
 * Provides both the V1 AEAD API and the streaming encrypt/decrypt API
 * for all 4 modes (Photon, Quantum, Chaos, Secure).
 *
 * Usage:
 *   const kelvin = require('./kelvin');
 *   const k = new kelvin.Kelvin(configJson);
 *   k.encrypt(data);
 *   k.close();
 *
 * Streaming:
 *   const enc = new kelvin.PhotonEncryptor(seed, 1000);
 *   const ct1 = enc.update(plaintext1);
 *   const ct2 = enc.update(plaintext2);
 *   const tag = enc.finalize();
 *   enc.close();
 */

const ffi = require('ffi-napi');
const ref = require('ref-napi');
const ArrayType = require('ref-array-di')(ref);

// Type definitions
const uint8 = ref.types.uint8;
const uint8Ptr = ref.refType(uint8);
const uint8Array = ArrayType(uint8);
const size_t = ref.types.size_t;
const size_tPtr = ref.refType(size_t);
const uint64 = ref.types.uint64;
const int32 = ref.types.int32;
const cstring = ref.types.CString;
const cstringPtr = ref.refType(cstring);
const voidPtr = ref.refType(ref.types.void);

// Load the shared library
const libPath = process.platform === 'win32'
    ? `${__dirname}/../../target/debug/kelvin_ffi.dll`
    : `${__dirname}/../../target/debug/libkelvin_ffi.so`;

const lib = ffi.Library(libPath, {
    // V1 AEAD
    'kelvin_new': [voidPtr, [cstring, cstringPtr]],
    'kelvin_encrypt': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_decrypt': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_remaining_bytes': [uint64, [voidPtr]],
    'kelvin_free': [ref.types.void, [voidPtr]],
    'kelvin_free_string': [ref.types.void, [cstring]],

    // Photon encryptor
    'kelvin_photon_encryptor_new': [voidPtr, [uint8Array, size_t, uint64, cstringPtr]],
    'kelvin_photon_encryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_photon_encryptor_finalize': [int32, [voidPtr, uint8Array, size_t, size_tPtr]],
    'kelvin_photon_encryptor_free': [ref.types.void, [voidPtr]],

    // Photon decryptor
    'kelvin_photon_decryptor_new': [voidPtr, [uint8Array, size_t, uint64, cstringPtr]],
    'kelvin_photon_decryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_photon_decryptor_finalize': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_photon_decryptor_free': [ref.types.void, [voidPtr]],

    // Quantum encryptor
    'kelvin_quantum_encryptor_new': [voidPtr, [uint8Array, size_t, uint64, cstringPtr]],
    'kelvin_quantum_encryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_quantum_encryptor_finalize': [int32, [voidPtr, uint8Array, size_t, size_tPtr]],
    'kelvin_quantum_encryptor_free': [ref.types.void, [voidPtr]],

    // Quantum decryptor
    'kelvin_quantum_decryptor_new': [voidPtr, [uint8Array, size_t, uint64, cstringPtr]],
    'kelvin_quantum_decryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_quantum_decryptor_finalize': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_quantum_decryptor_free': [ref.types.void, [voidPtr]],

    // Chaos encryptor
    'kelvin_chaos_encryptor_new': [voidPtr, [cstring, uint64, cstringPtr]],
    'kelvin_chaos_encryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_chaos_encryptor_finalize': [int32, [voidPtr, uint8Array, size_t, size_tPtr]],
    'kelvin_chaos_encryptor_free': [ref.types.void, [voidPtr]],

    // Chaos decryptor
    'kelvin_chaos_decryptor_new': [voidPtr, [cstring, uint64, cstringPtr]],
    'kelvin_chaos_decryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_chaos_decryptor_finalize': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_chaos_decryptor_free': [ref.types.void, [voidPtr]],

    // Secure encryptor
    'kelvin_secure_encryptor_new': [voidPtr, [uint8Array, size_t, uint8Array, size_t, cstringPtr]],
    'kelvin_secure_encryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_secure_encryptor_finalize': [int32, [voidPtr, uint8Array, size_t, size_tPtr]],
    'kelvin_secure_encryptor_free': [ref.types.void, [voidPtr]],

    // Secure decryptor
    'kelvin_secure_decryptor_new': [voidPtr, [uint8Array, size_t, uint8Array, size_t, cstringPtr]],
    'kelvin_secure_decryptor_update': [int32, [voidPtr, uint8Array, size_t, uint8Array, size_t, size_tPtr]],
    'kelvin_secure_decryptor_finalize': [int32, [voidPtr, uint8Array, size_t]],
    'kelvin_secure_decryptor_free': [ref.types.void, [voidPtr]],
});

// ============================================================================
// Error handling
// ============================================================================

class KelvinError extends Error {
    constructor(message) {
        super(message);
        this.name = 'KelvinError';
    }
}

function checkError(res, errorOut) {
    if (res !== 0) {
        const errStr = errorOut.deref();
        if (errStr) {
            const msg = errStr.readCString();
            lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
        throw new KelvinError('unknown error');
    }
}

// ============================================================================
// V1 Kelvin — AEAD encrypt/decrypt (existing)
// ============================================================================

class Kelvin {
    constructor(configJson) {
        const errorOut = ref.alloc(cstringPtr);
        this._ctx = lib.kelvin_new(configJson, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to initialize Kelvin';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    encrypt(data) {
        const buf = Buffer.from(data);
        const res = lib.kelvin_encrypt(this._ctx, buf, buf.length);
        if (res !== 0) throw new KelvinError('encryption failed');
        return buf;
    }

    decrypt(data) {
        const buf = Buffer.from(data);
        const res = lib.kelvin_decrypt(this._ctx, buf, buf.length);
        if (res !== 0) throw new KelvinError('decryption failed');
        return buf;
    }

    remainingBytes() {
        return lib.kelvin_remaining_bytes(this._ctx);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_free(this._ctx);
            this._ctx = null;
        }
    }
}

// ============================================================================
// Streaming API — Generic helpers
// ============================================================================

function streamingUpdate(libFn, ctx, input) {
    if (input.length === 0) return Buffer.alloc(0);
    const inputBuf = Buffer.from(input);
    const outputBuf = Buffer.alloc(input.length);
    const outputLen = ref.alloc(size_t);
    const res = libFn(ctx, inputBuf, inputBuf.length, outputBuf, outputBuf.length, outputLen);
    if (res !== 0) throw new KelvinError('streaming update failed');
    return outputBuf.slice(0, outputLen.deref());
}

function streamingFinalize(libFn, ctx) {
    const tagBuf = Buffer.alloc(64);
    const tagLen = ref.alloc(size_t);
    const res = libFn(ctx, tagBuf, tagBuf.length, tagLen);
    if (res !== 0) throw new KelvinError('streaming finalize failed');
    return tagBuf.slice(0, tagLen.deref());
}

function streamingDecryptFinalize(libFn, ctx, tag) {
    const tagBuf = tag ? Buffer.from(tag) : null;
    const res = libFn(ctx, tagBuf, tagBuf ? tagBuf.length : 0);
    if (res !== 0) throw new KelvinError('streaming decrypt finalize failed (tag mismatch)');
}

// ============================================================================
// Photon (V3) Streaming
// ============================================================================

class PhotonEncryptor {
    constructor(seed, maxReseeds = 1000) {
        const errorOut = ref.alloc(cstringPtr);
        const seedBuf = Buffer.from(seed);
        this._ctx = lib.kelvin_photon_encryptor_new(seedBuf, seedBuf.length, maxReseeds, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create PhotonEncryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(plaintext) {
        return streamingUpdate(lib.kelvin_photon_encryptor_update, this._ctx, plaintext);
    }

    finalize() {
        return streamingFinalize(lib.kelvin_photon_encryptor_finalize, this._ctx);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_photon_encryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

class PhotonDecryptor {
    constructor(seed, maxReseeds = 1000) {
        const errorOut = ref.alloc(cstringPtr);
        const seedBuf = Buffer.from(seed);
        this._ctx = lib.kelvin_photon_decryptor_new(seedBuf, seedBuf.length, maxReseeds, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create PhotonDecryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(ciphertext) {
        return streamingUpdate(lib.kelvin_photon_decryptor_update, this._ctx, ciphertext);
    }

    finalize(tag = null) {
        streamingDecryptFinalize(lib.kelvin_photon_decryptor_finalize, this._ctx, tag);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_photon_decryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

// ============================================================================
// Quantum (H) Streaming
// ============================================================================

class QuantumEncryptor {
    constructor(seed, maxReseeds = 1000) {
        const errorOut = ref.alloc(cstringPtr);
        const seedBuf = Buffer.from(seed);
        this._ctx = lib.kelvin_quantum_encryptor_new(seedBuf, seedBuf.length, maxReseeds, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create QuantumEncryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(plaintext) {
        return streamingUpdate(lib.kelvin_quantum_encryptor_update, this._ctx, plaintext);
    }

    finalize() {
        return streamingFinalize(lib.kelvin_quantum_encryptor_finalize, this._ctx);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_quantum_encryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

class QuantumDecryptor {
    constructor(seed, maxReseeds = 1000) {
        const errorOut = ref.alloc(cstringPtr);
        const seedBuf = Buffer.from(seed);
        this._ctx = lib.kelvin_quantum_decryptor_new(seedBuf, seedBuf.length, maxReseeds, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create QuantumDecryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(ciphertext) {
        return streamingUpdate(lib.kelvin_quantum_decryptor_update, this._ctx, ciphertext);
    }

    finalize(tag = null) {
        streamingDecryptFinalize(lib.kelvin_quantum_decryptor_finalize, this._ctx, tag);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_quantum_decryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

// ============================================================================
// Chaos (V2) Streaming
// ============================================================================

class ChaosEncryptor {
    constructor(configJson, bytesPerStep = 65536) {
        const errorOut = ref.alloc(cstringPtr);
        this._ctx = lib.kelvin_chaos_encryptor_new(configJson, bytesPerStep, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create ChaosEncryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(plaintext) {
        return streamingUpdate(lib.kelvin_chaos_encryptor_update, this._ctx, plaintext);
    }

    finalize() {
        return streamingFinalize(lib.kelvin_chaos_encryptor_finalize, this._ctx);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_chaos_encryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

class ChaosDecryptor {
    constructor(configJson, bytesPerStep = 65536) {
        const errorOut = ref.alloc(cstringPtr);
        this._ctx = lib.kelvin_chaos_decryptor_new(configJson, bytesPerStep, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create ChaosDecryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(ciphertext) {
        return streamingUpdate(lib.kelvin_chaos_decryptor_update, this._ctx, ciphertext);
    }

    finalize(tag = null) {
        streamingDecryptFinalize(lib.kelvin_chaos_decryptor_finalize, this._ctx, tag);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_chaos_decryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

// ============================================================================
// Secure (V1) Streaming
// ============================================================================

class SecureEncryptor {
    constructor(key, nonce) {
        if (key.length !== 32) throw new KelvinError('key must be 32 bytes');
        if (nonce.length !== 12) throw new KelvinError('nonce must be 12 bytes');
        const errorOut = ref.alloc(cstringPtr);
        const keyBuf = Buffer.from(key);
        const nonceBuf = Buffer.from(nonce);
        this._ctx = lib.kelvin_secure_encryptor_new(keyBuf, keyBuf.length, nonceBuf, nonceBuf.length, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create SecureEncryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(plaintext) {
        return streamingUpdate(lib.kelvin_secure_encryptor_update, this._ctx, plaintext);
    }

    finalize() {
        return streamingFinalize(lib.kelvin_secure_encryptor_finalize, this._ctx);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_secure_encryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

class SecureDecryptor {
    constructor(key, nonce) {
        if (key.length !== 32) throw new KelvinError('key must be 32 bytes');
        if (nonce.length !== 12) throw new KelvinError('nonce must be 12 bytes');
        const errorOut = ref.alloc(cstringPtr);
        const keyBuf = Buffer.from(key);
        const nonceBuf = Buffer.from(nonce);
        this._ctx = lib.kelvin_secure_decryptor_new(keyBuf, keyBuf.length, nonceBuf, nonceBuf.length, errorOut);
        if (this._ctx.isNull()) {
            const errStr = errorOut.deref();
            const msg = errStr ? errStr.readCString() : 'failed to create SecureDecryptor';
            if (errStr) lib.kelvin_free_string(errStr);
            throw new KelvinError(msg);
        }
    }

    update(ciphertext) {
        return streamingUpdate(lib.kelvin_secure_decryptor_update, this._ctx, ciphertext);
    }

    finalize(tag) {
        streamingDecryptFinalize(lib.kelvin_secure_decryptor_finalize, this._ctx, tag);
    }

    close() {
        if (this._ctx) {
            lib.kelvin_secure_decryptor_free(this._ctx);
            this._ctx = null;
        }
    }
}

// ============================================================================
// Exports
// ============================================================================

module.exports = {
    KelvinError,
    Kelvin,
    PhotonEncryptor,
    PhotonDecryptor,
    QuantumEncryptor,
    QuantumDecryptor,
    ChaosEncryptor,
    ChaosDecryptor,
    SecureEncryptor,
    SecureDecryptor,
};
