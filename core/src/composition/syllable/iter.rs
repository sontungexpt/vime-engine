/// Iterator over rendered syllable characters, handling both building and dead phases.
pub(crate) enum SyllableChars<I1, I2> {
    Building(I1),
    Dead(I2),
}

impl<I1, I2> Iterator for SyllableChars<I1, I2>
where
    I1: Iterator<Item = char>,
    I2: Iterator<Item = char>,
{
    type Item = char;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Building(i) => i.next(),
            Self::Dead(i) => i.next(),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Building(i) => i.size_hint(),
            Self::Dead(i) => i.size_hint(),
        }
    }
}