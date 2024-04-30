import { ReactElement } from "react";
import { HeaderMenuItemsDefinition } from "../types";
import { Link } from 'react-router-dom';

export default class NavigationMenuBuilder {
    static buildMenuItems(definition: HeaderMenuItemsDefinition): ReactElement[] {
        if (!definition.items.length) {
            return []; // or error
        }

        return definition.items.map((item, i) => {
            switch (item.type) {
                case 'button-link': {
                    return <Link key={i} to={item.navigationUrl}><button>{item.textContent}</button></Link>
                }
                default: { return <div>{item.textContent}</div> }
            }
        })
    }
}