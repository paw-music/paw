use num::FromPrimitive;

macro_rules! notes {
    ($($name: ident: $freq: expr),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
        #[repr(u8)]
        pub enum Note {
            $($name),*
        }

        impl Note {
            #[inline]
            pub fn each() -> impl Iterator<Item = Note> {
                const NOTES: [Note; 128] = [$(Note::$name),*];

                NOTES.iter().copied()
            }

            #[inline]
            pub const fn freq(&self) -> crate::osc::clock::Freq {
                match self {
                    $(Self::$name => crate::osc::clock::Freq::new($freq)),*
                }
            }

            // TODO: FromPrimitive or don't even use this
            // pub const fn from_midi(midi_note: u8) -> Self {
            //     unsafe {core::mem::transmute(midi_note)}
            // }
        }
    };
}

impl Note {
    #[inline]
    pub fn saturating_add(self, transpose: i16) -> Self {
        FromPrimitive::from_i16((self as i16).saturating_add(transpose).clamp(0, 127)).unwrap()
    }
}

notes! {
    Cmin1: 8.18,
    Csmin1: 8.66,
    Dmin1: 9.18,
    Dsmin1: 9.72,
    Emin1: 10.30,
    Fmin1: 10.91,
    Fsmin1: 11.56,
    Gmin1: 12.25,
    Gsmin1: 12.98,
    Amin1: 13.75,
    Asmin1: 14.57,
    Bmin1: 15.43,
    C0: 16.35,
    Cs0: 17.32,
    D0: 18.35,
    Ds0: 19.45,
    E0: 20.60,
    F0: 21.83,
    Fs0: 23.12,
    G0: 24.50,
    Gs0: 25.96,
    A0: 27.50,
    As0: 29.14,
    B0: 30.87,
    C1: 32.70,
    Cs1: 34.65,
    D1: 36.71,
    Ds1: 38.89,
    E1: 41.20,
    F1: 43.65,
    Fs1: 46.25,
    G1: 49.00,
    Gs1: 51.91,
    A1: 55.00,
    As1: 58.27,
    B1: 61.74,
    C2: 65.41,
    Cs2: 69.30,
    D2: 73.42,
    Ds2: 77.78,
    E2: 82.41,
    F2: 87.31,
    Fs2: 92.50,
    G2: 98.00,
    Gs2: 103.83,
    A2: 110.00,
    As2: 116.54,
    B2: 123.47,
    C3: 130.81,
    Cs3: 138.59,
    D3: 146.83,
    Ds3: 155.56,
    E3: 164.81,
    F3: 174.61,
    Fs3: 185.00,
    G3: 196.00,
    Gs3: 207.65,
    A3: 220.00,
    As3: 233.08,
    B3: 246.94,
    C4: 261.63,
    Cs4: 277.18,
    D4: 293.66,
    Ds4: 311.13,
    E4: 329.63,
    F4: 349.23,
    Fs4: 369.99,
    G4: 392.00,
    Gs4: 415.30,
    A4: 440.00,
    As4: 466.16,
    B4: 493.88,
    C5: 523.25,
    Cs5: 554.37,
    D5: 587.33,
    Ds5: 622.25,
    E5: 659.26,
    F5: 698.46,
    Fs5: 739.99,
    G5: 783.99,
    Gs5: 830.61,
    A5: 880.00,
    As5: 932.33,
    B5: 987.77,
    C6: 1046.50,
    Cs6: 1108.73,
    D6: 1174.66,
    Ds6: 1244.51,
    E6: 1318.51,
    F6: 1396.91,
    Fs6: 1479.98,
    G6: 1567.98,
    Gs6: 1661.22,
    A6: 1760.00,
    As6: 1864.65,
    B6: 1975.53,
    C7: 2093.00,
    Cs7: 2217.46,
    D7: 2349.32,
    Ds7: 2489.02,
    E7: 2637.02,
    F7: 2793.83,
    Fs7: 2959.96,
    G7: 3135.96,
    Gs7: 3322.44,
    A7: 3520.00,
    As7: 3729.31,
    B7: 3951.07,
    C8: 4186.01,
    Cs8: 4434.92,
    D8: 4698.64,
    Ds8: 4978.03,
    E8: 5274.04,
    F8: 5587.65,
    Fs8: 5919.91,
    G8: 6271.93,
    Gs8: 6644.88,
    A8: 7040.00,
    As8: 7458.62,
    B8: 7902.13,
    C9: 8372.02,
    Cs9: 8869.84,
    D9: 9397.27,
    Ds9: 9956.06,
    E9: 10548.08,
    F9: 11175.30,
    Fs9: 11839.82,
    G9: 12543.86,
}

// /// Physical key
// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// pub struct Key(u8);
