export interface HeaderMenuItemsDefinition { 
    items: Array<HeaderMenuItem>;
};

export interface HeaderMenuItem {
    textContent: string;
    type: string;
    navigationUrl: string;
}