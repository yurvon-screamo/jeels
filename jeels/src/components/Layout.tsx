import { PropsWithChildren } from "react";
import { TitleBar, Frame, TaskBar, List } from "@react95/core";
import { Logo } from "@react95/icons";

export type LayoutProps = PropsWithChildren<{
    title: string;
    startMenu?: React.ReactElement;
}>;

export function Layout({ title, startMenu, children }: LayoutProps) {
    return (
        <div style={{
            background: 'var(--background)',
            minHeight: '100vh',
            display: 'flex',
            flexDirection: 'column'
        }}>
            <TitleBar title={title} icon={<Logo variant="32x32_4" />} active>
                <TitleBar.OptionsBox>
                    <TitleBar.Minimize />
                    <TitleBar.Maximize />
                    <TitleBar.Close />
                </TitleBar.OptionsBox>
            </TitleBar>

            <div style={{ flex: 1, display: 'flex', justifyContent: 'center', padding: 20 }}>
                <Frame w="100%" style={{ background: 'var(--window-bg)', border: '2px outset var(--window-border)', padding: 20 }}>
                    {children}
                </Frame>
            </div>

            <TaskBar list={startMenu ?? <List />} />
        </div>
    );
}


