import "./poker-card.css"
import heartsImage from '../../assets/symbols/hearts.png';
import spadesImage from '../../assets/symbols/spades.png';
import clubsImage from '../../assets/symbols/clubs.png';
import dimondsImage from '../../assets/symbols/diamonds.png';
import backImage from '../../assets/symbols/back.png';
import kingImage from '../../assets/symbols/king.png';
import jackImage from '../../assets/symbols/jack.png';
import queenImage from '../../assets/symbols/queen.png';


import { CardSuit, CardValue } from "../../types";

interface PokerCardProps {
    cardValue: CardValue;
    cardSuit: CardSuit;
}
function PokerCard({ cardSuit, cardValue }: PokerCardProps) {
    return <>
        <div className="poker-card">
            {cardSuit !== CardSuit.Empty &&
                <div className="poker-card-value" style={{ color: getCardColor(cardSuit) }}>{cardValue}</div>
            }
            <div className="poker-card-image">
                <img src={getCardImage(cardSuit, cardValue)} className="poker-card-image"></img>
            </div>
        </div>
    </>

    function getCardColor(cardSuit: CardSuit): string {
        switch (cardSuit) {
            case CardSuit.Clubs:
            case CardSuit.Spades: {
                return "black";
            }
            case CardSuit.Diamonds:
            case CardSuit.Hearts: {
                return "red";
            }
            default: return "";
        }
    }
    function getCardImage(cardSuit: CardSuit, cardValue: CardValue): string {
        switch (cardValue) {
            case "K": {
                return kingImage;
            }
            case "J": {
                return jackImage;
            }
            case "Q": {
                return queenImage;
            }
        }
        switch (cardSuit) {
            case CardSuit.Clubs: {
                return clubsImage;
            }
            case CardSuit.Diamonds: {
                return dimondsImage;
            }
            case CardSuit.Hearts: {
                return heartsImage;
            }
            case CardSuit.Spades: {
                return spadesImage;
            }
            default: return backImage;
        }
    }
}

export default PokerCard;