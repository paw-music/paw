## Dev notes

### Components

Components are parts such as LFOs, Envelopes, Oscillators, etc.

Most synthesizer components are implemented in the manner of `state` + `params`. The `state` is stored inside component instance but `params` are passed to its `tick` method. This is done to avoid storing same data in all instances and to avoid parameters synchronization (we set same properties for all components of one type and when we get them which instance is used? First instance?). But this is only about properties (`params`) that are common for all instances, for example each voice has its own frequency so it is possible to do detune spread.
