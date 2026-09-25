INSERT INTO settings (name, value, default_value)
VALUES ('tutorial_speak_bar_shown', 'false', 'false')
ON CONFLICT(name) DO UPDATE SET
    value = 'false',
    default_value = 'false';
