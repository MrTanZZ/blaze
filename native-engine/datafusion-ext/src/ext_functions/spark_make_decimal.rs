// Copyright 2022 The Blaze Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use datafusion::arrow::array::*;
use datafusion::common::Result;
use datafusion::common::ScalarValue;
use datafusion::physical_plan::ColumnarValue;
use std::sync::Arc;

/// implements org.apache.spark.sql.catalyst.expressions.MakeDecimal
pub fn spark_make_decimal(args: &[ColumnarValue]) -> Result<ColumnarValue> {
    let precision = match &args[1] {
        &ColumnarValue::Scalar(ScalarValue::Int32(Some(precision))) => precision as u8,
        _ => unreachable!("make_decimal.precision is not int32 value"),
    };
    let scale = match &args[2] {
        &ColumnarValue::Scalar(ScalarValue::Int32(Some(scale))) => scale as u8,
        _ => unreachable!("make_decimal.scale is not int32 value"),
    };
    assert!(
        precision >= 1,
        "make_decimal: illegal precision: {}",
        precision
    );

    Ok(match &args[0] {
        ColumnarValue::Scalar(scalar) => match scalar {
            ScalarValue::Int64(Some(v)) => ColumnarValue::Scalar(
                ScalarValue::Decimal128(Some(*v as i128), precision, scale),
            ),
            _ => ColumnarValue::Scalar(ScalarValue::Decimal128(None, precision, scale)),
        },
        ColumnarValue::Array(array) => {
            let array = array.as_any().downcast_ref::<Int64Array>().unwrap();
            let mut output = Decimal128Builder::with_capacity(array.len());

            for v in array.into_iter() {
                match v {
                    Some(v) => output.append_value(v as i128),
                    None => output.append_null(),
                }
            }
            ColumnarValue::Array(Arc::new(
                output.finish().with_precision_and_scale(precision, scale)?,
            ))
        }
    })
}
