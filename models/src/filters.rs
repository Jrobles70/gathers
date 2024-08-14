pub struct CardSearchFilters {
    pub card_name: Option<String>,
    pub card_colours: Option<Vec<CardColour>>,
}

pub enum CardColour {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colourless,
    Multicoloured
}
