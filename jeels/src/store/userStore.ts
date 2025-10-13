import { UserState } from './types';

type Listener = () => void;

const STORAGE_KEY = 'jeels.currentUser.v1';

function loadFromStorage(): UserState | null {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return null;
        const parsed = JSON.parse(raw) as UserState;
        if (!parsed || typeof parsed.id !== 'string') return null;
        if (!Array.isArray(parsed.learningLessonKeys)) parsed.learningLessonKeys = [];
        return parsed;
    } catch {
        return null;
    }
}

function saveToStorage(state: UserState) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {
        // ignore storage errors
    }
}

class UserStoreImpl {
    private state: UserState =
        loadFromStorage() ?? { id: 'local', name: 'Local User', learningLessonKeys: [] };
    private listeners: Set<Listener> = new Set();

    subscribe(listener: Listener): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    getState(): UserState {
        return this.state;
    }

    private notify() {
        this.listeners.forEach((l) => l());
    }

    setName(name: string) {
        this.state = { ...this.state, name };
        saveToStorage(this.state);
        this.notify();
    }

    addLesson(key: string) {
        if (this.state.learningLessonKeys.includes(key)) return;
        this.state = {
            ...this.state,
            learningLessonKeys: [...this.state.learningLessonKeys, key],
        };
        saveToStorage(this.state);
        this.notify();
    }

    removeLesson(key: string) {
        if (!this.state.learningLessonKeys.includes(key)) return;
        this.state = {
            ...this.state,
            learningLessonKeys: this.state.learningLessonKeys.filter((k) => k !== key),
        };
        saveToStorage(this.state);
        this.notify();
    }
}

export const UserStore = new UserStoreImpl();


