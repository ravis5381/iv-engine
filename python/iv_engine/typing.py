"""Typing aliases for the public Python API."""

from __future__ import annotations

from typing import TYPE_CHECKING, Union

import numpy as np
from numpy.typing import NDArray

if TYPE_CHECKING:
    import pandas as pd

Float = float
FloatArray = NDArray[np.float64]
FloatOrArray = Union[float, FloatArray, "pd.Series"]

__all__ = ["Float", "FloatArray", "FloatOrArray"]
