CREATE TABLE IF NOT EXISTS ListenerFrame (
    Listener INT,
    Frame INT,
    PRIMARY KEY (Listener, Frame)
);
CREATE INDEX IF NOT EXISTS ListenerFrame_Listener ON ListenerFrame (Listener);
CREATE INDEX IF NOT EXISTS ListenerFrame_Frame ON ListenerFrame (Frame ASC);