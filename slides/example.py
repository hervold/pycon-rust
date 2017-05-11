import ctypes

ex_lib = ctypes.CDLL("target/release/libtwist_pycon_slides.dylib")

ex_lib.fast_inv_sqrt.restype = ctypes.c_float
ex_lib.fast_inv_sqrt.argtypes = [ctypes.c_float]

print ex_lib.fast_inv_sqrt( 4.0 )
print ex_lib.fast_inv_sqrt( 9.0 )
