from setuptools import setup, find_packages

setup(
    name="kelvin-py",
    version="0.1.0",
    description="Python bindings for the Kelvin Orbital Chaos KDF Cryptosystem",
    author="Nicolas Liaudat",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    python_requires=">=3.7",
    install_requires=[],
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Topic :: Security :: Cryptography",
        "Programming Language :: Python :: 3",
    ],
)
