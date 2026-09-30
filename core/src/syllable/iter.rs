/// Iterator over a [`SyllableChars`](super::SyllableChars), handling both
/// the building and the dead phase without materialising the buffer first.
pub(crate) enum SyllableCharsIter<I1, I2> {
    Building(I1),
    Dead(I2),
}

impl<I1, I2> Iterator for SyllableCharsIter<I1, I2>
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
