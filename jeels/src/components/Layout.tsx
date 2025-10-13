import { PropsWithChildren } from "react";
import { TaskBar, List } from "@react95/core";

export type LayoutProps = PropsWithChildren<{
    title: string;
    startMenu?: React.ReactElement;
}>;

export function Layout({ startMenu, children }: LayoutProps) {
    return (
        <div style={{
            background: 'var(--background)',
            minHeight: '100vh',
            display: 'flex',
            flexDirection: 'column'
        }}>
            <div style={{ flex: 1, position: 'relative', padding: 0, overflow: 'hidden' }}>
                {children}
            </div>

            <TaskBar list={startMenu ?? <List />} />
        </div>
    );
}


