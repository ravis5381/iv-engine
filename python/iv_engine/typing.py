"""Typing aliases for the public Python API."""

from __future__ import annotations

from typing import Union

import numpy as np
from numpy.typing import NDArray

Float = float
FloatArray = NDArray[np.float64]
FloatOrArray = Union[float, FloatArray]

__all__ = ["Float", "FloatArray", "FloatOrArray"]
