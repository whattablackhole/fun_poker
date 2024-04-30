import { HeaderMenuItemsDefinition } from "../../types";
import NavigationMenuBuilder from "../../utils/navigation-menu-builder";



export default function NavigationHeader() {
    const headerMenuItemsDefinition: HeaderMenuItemsDefinition = { items: [{ textContent: 'Create New Lobby', type: 'button-link', navigationUrl: '/new-lobby' }] };

    const menuItems = NavigationMenuBuilder.buildMenuItems(headerMenuItemsDefinition);

    return <div style={{ width: '100%', height: "40px", display: 'flex', alignItems: 'center', justifyContent: 'left', gap: "10px" }}>
        {menuItems}
    </div>
} 