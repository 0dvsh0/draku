# paddle-ocr-rs, vendored

This directory is `paddle-ocr-rs` 0.6.1, Apache-2.0, from
<https://github.com/mg-chao/paddle-ocr-rs>. The upstream license is in
`LICENSE`; nothing here changes it.

## Why it is vendored

Upstream 0.6.1 pins `ort` 2.0.0-rc.10. Draku's embedding model
(EmbeddingGemma-300M Q4 from `fastembed`) needs `ort` 2.0.0-rc.13, and Cargo can
only link one ONNX Runtime version. The patch is limited to the two API changes
between those releases:

- `session.run(...)` now takes `inputs![name => Tensor]`;
- tensor extraction returns the newer `(shape, data)` form;
- `SessionBuilder::commit_from_memory` / `commit_from_file` keep the same
  semantics with the rc.13 signatures.

No detection, angle or recognition logic was changed. The OCR models beside the
app (`det.onnx`, `cls.onnx`, `rec.onnx`, `dict.txt`) are PaddleOCR PP-OCR
models, Apache-2.0.
