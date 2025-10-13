// Domain types for Lessons and User

export type LessonGroup = string; // e.g., '動詞', '形容詞'

export interface LessonMeta {
    group: LessonGroup;
    topic: string; // folder name and display title, e.g., '待ちます, 探します, 見つけます'
}

export interface LessonYaml {
    topic: string;
    general_img_promt?: string;
    general_md_content: string;
    practic_img_promt?: string;
    practic_md_content: string;
}

// YAML index shapes
export interface TopIndexYaml {
    groups?: string[]; // e.g., ['動詞', '形容詞']
    lessons?: { group: string; topic: string }[]; // optional alternative shape
}

export interface GroupIndexYaml {
    topics?: string[]; // e.g., ['待ちます, 探します, 見つけます']
    lessons?: { topic: string }[]; // optional alternative shape
}

export interface LessonContent {
    group: LessonGroup;
    topic: string;
    // computed asset paths (served from public/content)
    general_audio_path: string;
    general_img_path: string;
    general_md_content: string;
    practic_audio_path: string;
    practic_img_path: string;
    practic_md_content: string;
    // optional prompts that produced the images
    general_img_promt?: string;
    practic_img_promt?: string;
}

export interface LessonsState {
    index: LessonMeta[];
    lessonsByKey: Record<string, LessonContent | undefined>;
    errorsByKey: Record<string, string | undefined>;
    loadingKeys: Set<string>;
}

export interface UserState {
    id: string;
    name: string;
    learningLessonKeys: string[]; // keys referencing lessons in LessonsState
}

export type Selector<TState, TSlice> = (state: TState) => TSlice;


