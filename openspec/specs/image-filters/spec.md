# Image Filters

## Purpose
Define the per-image filter chain applied after transform and before watermark.

## Requirements

### Requirement: Chain syntax
The service MUST accept a `filters` parameter whose value is a colon-separated
list of `name` or `name(arg)` invocations.

#### Scenario: empty chain
- GIVEN an empty `filters` value
- WHEN the handler parses it
- THEN the resulting filter chain is empty and no filters are applied

#### Scenario: full chain
- GIVEN `filters=grayscale:brightness(20):blur(2.0)`
- WHEN the handler parses it
- THEN the chain is three filters applied in that order

### Requirement: Filter ordering
The service MUST apply filters in the order they appear in the chain.

#### Scenario: order preserved
- GIVEN `filters=grayscale:brightness(20)` and a non-trivial input image
- WHEN the handler applies the chain
- THEN the result is the input first converted to grayscale and then
  brightened (not the other way around)

### Requirement: Built-in filters
The service MUST support at least these filters: `grayscale`, `brightness(N)`,
`contrast(N)`, `blur(sigma)`.

#### Scenario: each built-in filter applies
- GIVEN any of `grayscale`, `brightness(20)`, `contrast(20)`, `blur(2.0)`
- WHEN the handler applies that single filter to a non-trivial input
- THEN the output image bytes are different from the input bytes

### Requirement: Argument validation
The service MUST validate each filter's argument count and parseability, and
reject the request with status 400 and `err.kind` of `invalid_filter` on failure.

#### Scenario: unknown filter
- GIVEN `filters=sepia`
- WHEN the handler parses it
- THEN the response status is 400 and `err.kind` is `invalid_filter`

#### Scenario: bad arg count
- GIVEN `filters=brightness(1,2)`
- WHEN the handler parses it
- THEN the response status is 400 and `err.kind` is `invalid_filter`

#### Scenario: out of range blur
- GIVEN `filters=blur(500)` (above the allowed ceiling)
- WHEN the handler parses it
- THEN the response status is 400 and `err.kind` is `invalid_filter`
