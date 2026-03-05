# axcli-command-matrix

| Command | Description | Status | axcli.rs Alignment |
|---------|-------------|--------|-------------------|
| snapshot | Take a snapshot of the UI tree | Implemented (mock-backed + AT-SPI adapter scaffold) | Yes |
| click | Click an element | Implemented | Yes |
| dblclick | Double click an element | Implemented | Yes |
| input | Input text to an element | Implemented (focus fallback enabled) | Yes |
| fill | Fill text to an element (clear first) | Implemented | Yes |
| press | Press a key | Implemented | Yes |
| hover | Hover over an element | Implemented | Yes |
| focus | Focus an element | Implemented | Yes |
| scroll-to | Scroll to an element | Implemented | Yes |
| scroll | Scroll in a direction | Implemented (strict direction validation) | Yes |
| screenshot | Take a screenshot | Implemented (temp file cleanup + sensitive-node guard) | Yes |
| wait | Wait for an element or condition | Implemented | Yes |
| get | Get property of an element | Implemented (sensitive-node guard) | Yes |
| list-apps | List running accessible apps | Implemented | Yes |
