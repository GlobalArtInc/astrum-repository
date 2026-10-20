UPDATE setting
SET value = jsonb_set(value, '{title}', '"Astrum Repository"'), updated_at = now()
WHERE key = 'ui.settings' AND value ->> 'title' = 'GlobalArt Astrum Repository';
