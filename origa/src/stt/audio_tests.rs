use super::audio::*;

#[test]
fn downmix_to_mono_stereo() {
    let stereo = vec![1.0, 0.5, -1.0, -0.5, 0.0, 1.0];
    let mono = downmix_to_mono(&stereo, 2);
    assert_eq!(mono.len(), 3);
    assert!((mono[0] - 0.75).abs() < 0.001);
    assert!((mono[1] - (-0.75)).abs() < 0.001);
    assert!((mono[2] - 0.5).abs() < 0.001);
}

#[test]
fn downmix_to_mono_already_mono() {
    let mono = vec![1.0, 2.0, 3.0];
    let result = downmix_to_mono(&mono, 1);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn resample_same_rate() {
    let samples = vec![1.0, 2.0, 3.0, 4.0];
    let result = resample(&samples, 16000);
    assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn resample_downsample() {
    let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let result = resample(&samples, 48000);
    assert_eq!(result.len(), 2);
    assert!((result[0] - 0.0).abs() < 0.001);
    assert!((result[1] - 3.0).abs() < 0.001);
}

#[test]
fn resample_upsample() {
    let samples = vec![0.0, 2.0, 4.0];
    let result = resample(&samples, 8000);
    assert_eq!(result.len(), 6);
    assert!((result[0] - 0.0).abs() < 0.001);
    assert!((result[1] - 1.0).abs() < 0.001);
    assert!((result[2] - 2.0).abs() < 0.001);
}

#[test]
fn pad_or_trim_trim() {
    let samples = vec![1.0; 100];
    let result = pad_or_trim(&samples, 50);
    assert_eq!(result.len(), 50);
}

#[test]
fn pad_or_trim_pad() {
    let samples = vec![1.0, 2.0, 3.0];
    let result = pad_or_trim(&samples, 10);
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[2], 3.0);
    assert_eq!(result[3], 0.0);
    assert_eq!(result[9], 0.0);
}

#[test]
fn pad_or_trim_exact() {
    let samples = vec![1.0, 2.0, 3.0];
    let result = pad_or_trim(&samples, 3);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}
