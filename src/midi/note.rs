use num::FromPrimitive;

macro_rules! notes {
    ($($name: ident: $freq: expr),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
        #[repr(u8)]
        pub enum Note {
            $($name),*
        }

        impl Note {
            pub fn each() -> impl Iterator<Item = Note> {
                const NOTES: [Note; 128] = [$(Note::$name),*];

                NOTES.iter().copied()
            }

            // pub fn from_freq(freq)

            pub fn freq(&self) -> f32 {
                match self {
                    $(Self::$name => $freq),*
                }
            }
        }
    };
}

impl Note {
    pub fn saturating_add(self, transpose: i16) -> Self {
        FromPrimitive::from_i16((self as i16).saturating_add(transpose).clamp(0, 127)).unwrap()
    }
}

notes! {
    Cmin2: 8.18,
    Csmin2: 8.66,
    Dmin2: 9.18,
    Dsmin2: 9.72,
    Emin2: 10.30,
    Fmin2: 10.91,
    Fsmin2: 11.56,
    Gmin2: 12.25,
    Gsmin2: 12.98,
    Amin2: 13.75,
    Asmin2: 14.57,
    Bmin2: 15.43,
    Cmin1: 16.35,
    Csmin1: 17.32,
    Dmin1: 18.35,
    Dsmin1: 19.45,
    Emin1: 20.60,
    Fmin1: 21.83,
    Fsmin1: 23.12,
    Gmin1: 24.50,
    Gsmin1: 25.96,
    Amin1: 27.50,
    Asmin1: 29.14,
    Bmin1: 30.87,
    C0: 32.70,
    Cs0: 34.65,
    D0: 36.71,
    Ds0: 38.89,
    E0: 41.20,
    F0: 43.65,
    Fs0: 46.25,
    G0: 49.00,
    Gs0: 51.91,
    A0: 55.00,
    As0: 58.27,
    B0: 61.74,
    C1: 65.41,
    Cs1: 69.30,
    D1: 73.42,
    Ds1: 77.78,
    E1: 82.41,
    F1: 87.31,
    Fs1: 92.50,
    G1: 98.00,
    Gs1: 103.83,
    A1: 110.00,
    As1: 116.54,
    B1: 123.47,
    C2: 130.81,
    Cs2: 138.59,
    D2: 146.83,
    Ds2: 155.56,
    E2: 164.81,
    F2: 174.61,
    Fs2: 185.00,
    G2: 196.00,
    Gs2: 207.65,
    A2: 220.00,
    As2: 233.08,
    B2: 246.94,
    C3: 261.63,
    Cs3: 277.18,
    D3: 293.66,
    Ds3: 311.13,
    E3: 329.63,
    F3: 349.23,
    Fs3: 369.99,
    G3: 392.00,
    Gs3: 415.30,
    A3: 440.00,
    As3: 466.16,
    B3: 493.88,
    C4: 523.25,
    Cs4: 554.37,
    D4: 587.33,
    Ds4: 622.25,
    E4: 659.26,
    F4: 698.46,
    Fs4: 739.99,
    G4: 783.99,
    Gs4: 830.61,
    A4: 880.00,
    As4: 932.33,
    B4: 987.77,
    C5: 1046.50,
    Cs5: 1108.73,
    D5: 1174.66,
    Ds5: 1244.51,
    E5: 1318.51,
    F5: 1396.91,
    Fs5: 1479.98,
    G5: 1567.98,
    Gs5: 1661.22,
    A5: 1760.00,
    As5: 1864.65,
    B5: 1975.53,
    C6: 2093.00,
    Cs6: 2217.46,
    D6: 2349.32,
    Ds6: 2489.02,
    E6: 2637.02,
    F6: 2793.83,
    Fs6: 2959.96,
    G6: 3135.96,
    Gs6: 3322.44,
    A6: 3520.00,
    As6: 3729.31,
    B6: 3951.07,
    C7: 4186.01,
    Cs7: 4434.92,
    D7: 4698.64,
    Ds7: 4978.03,
    E7: 5274.04,
    F7: 5587.65,
    Fs7: 5919.91,
    G7: 6271.93,
    Gs7: 6644.88,
    A7: 7040.00,
    As7: 7458.62,
    B7: 7902.13,
    C8: 8372.02,
    Cs8: 8869.85,
    D8: 9397.27,
    Ds8: 9956.06,
    E8: 10548.08,
    F8: 11175.30,
    Fs8: 11839.82,
    G8: 12543.86,
}

// /// Physical key
// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// pub struct Key(u8);
