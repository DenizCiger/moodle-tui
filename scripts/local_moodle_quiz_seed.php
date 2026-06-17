<?php
define('CLI_SCRIPT', true);

require_once('/bitnami/moodle/config.php');
require_once($CFG->libdir . '/clilib.php');
require_once($CFG->dirroot . '/user/lib.php');
require_once($CFG->dirroot . '/course/lib.php');
require_once($CFG->dirroot . '/course/modlib.php');
require_once($CFG->dirroot . '/enrol/locallib.php');
require_once($CFG->dirroot . '/mod/quiz/lib.php');
require_once($CFG->dirroot . '/mod/quiz/locallib.php');
require_once($CFG->dirroot . '/question/editlib.php');

global $DB, $CFG;

$trace = new text_progress_trace();

function ensure_config_enabled(string $name, string $value): void {
    set_config($name, $value);
}

function ensure_student(): stdClass {
    global $DB;
    $user = $DB->get_record('user', ['username' => 'student', 'deleted' => 0]);
    if ($user) {
        return $user;
    }
    $user = (object)[
        'auth' => 'manual',
        'confirmed' => 1,
        'mnethostid' => 1,
        'username' => 'student',
        'firstname' => 'TUI',
        'lastname' => 'Student',
        'email' => 'student@example.test',
        'password' => hash_internal_user_password('studentpass'),
        'timecreated' => time(),
        'timemodified' => time(),
    ];
    $user->id = user_create_user($user, false, false);
    return $DB->get_record('user', ['id' => $user->id], '*', MUST_EXIST);
}

function ensure_course(): stdClass {
    global $DB;
    $course = $DB->get_record('course', ['shortname' => 'TUI-QUIZ']);
    if ($course) {
        return $course;
    }
    $category = $DB->get_record('course_categories', ['id' => 1], '*', MUST_EXIST);
    $data = (object)[
        'fullname' => 'Moodle TUI Quiz Test Course',
        'shortname' => 'TUI-QUIZ',
        'category' => $category->id,
        'visible' => 1,
        'format' => 'topics',
        'numsections' => 1,
        'summary' => 'Local test course for moodle-tui quiz attempts.',
    ];
    return create_course($data);
}

function ensure_enrolment(stdClass $course, stdClass $user): void {
    global $DB;
    $instances = enrol_get_instances($course->id, true);
    $manual = null;
    foreach ($instances as $instance) {
        if ($instance->enrol === 'manual') {
            $manual = $instance;
            break;
        }
    }
    if (!$manual) {
        $plugin = enrol_get_plugin('manual');
        $manualid = $plugin->add_instance($course);
        $manual = $DB->get_record('enrol', ['id' => $manualid], '*', MUST_EXIST);
    }
    $context = context_course::instance($course->id);
    $studentrole = $DB->get_record('role', ['shortname' => 'student'], '*', MUST_EXIST);
    if (!is_enrolled($context, $user, '', true)) {
        enrol_get_plugin('manual')->enrol_user($manual, $user->id, $studentrole->id);
    }
}

function ensure_mobile_access(): void {
    global $DB;
    ensure_config_enabled('enablewebservices', '1');
    ensure_config_enabled('enablemobilewebservice', '1');
    ensure_config_enabled('webserviceprotocols', 'rest');
    $service = $DB->get_record('external_services', ['shortname' => 'moodle_mobile_app']);
    if ($service && (!$service->enabled || $service->restrictedusers)) {
        $service->enabled = 1;
        $service->restrictedusers = 0;
        $DB->update_record('external_services', $service);
    }
    $systemcontext = context_system::instance();
    $studentrole = $DB->get_record('role', ['shortname' => 'student'], '*', MUST_EXIST);
    $userrole = $DB->get_record('role', ['shortname' => 'user'], '*', MUST_EXIST);
    assign_capability('webservice/rest:use', CAP_ALLOW, $studentrole->id, $systemcontext->id, true);
    assign_capability('moodle/webservice:createtoken', CAP_ALLOW, $studentrole->id, $systemcontext->id, true);
    assign_capability('webservice/rest:use', CAP_ALLOW, $userrole->id, $systemcontext->id, true);
    assign_capability('moodle/webservice:createtoken', CAP_ALLOW, $userrole->id, $systemcontext->id, true);
}

function ensure_quiz(stdClass $course): stdClass {
    global $DB;
    $existing = $DB->get_record('quiz', ['course' => $course->id, 'name' => 'TUI supported questions quiz']);
    if ($existing) {
        if ($cm = $DB->get_record('course_modules', ['course' => $course->id, 'module' => $DB->get_field('modules', 'id', ['name' => 'quiz']), 'instance' => $existing->id])) {
            $existing->cmid = $cm->id;
            return $existing;
        }
        $DB->delete_records('quiz', ['id' => $existing->id]);
    }
    $module = $DB->get_record('modules', ['name' => 'quiz'], '*', MUST_EXIST);
    $section = course_create_section($course, 1);
    $quiz = (object)[
        'course' => $course->id,
        'name' => 'TUI supported questions quiz',
        'intro' => '<p>Seeded quiz for moodle-tui. Tests the AI model across multiple domains: computer science, mathematics, history, and general knowledge.</p>',
        'introformat' => FORMAT_HTML,
        'timeopen' => 0,
        'timeclose' => 0,
        'timelimit' => 0,
        'overduehandling' => 'autosubmit',
        'graceperiod' => 0,
        'preferredbehaviour' => 'deferredfeedback',
        'canredoquestions' => 0,
        'attempts' => 0,
        'attemptonlast' => 0,
        'grademethod' => QUIZ_GRADEHIGHEST,
        'decimalpoints' => 2,
        'questiondecimalpoints' => -1,
        'reviewattempt' => 0x11110,
        'reviewcorrectness' => 0x10000,
        'reviewmarks' => 0x11110,
        'reviewspecificfeedback' => 0x10000,
        'reviewgeneralfeedback' => 0x10000,
        'reviewrightanswer' => 0x10000,
        'reviewoverallfeedback' => 0x11110,
        'questionsperpage' => 1,
        'navmethod' => QUIZ_NAVMETHOD_FREE,
        'shuffleanswers' => 1,
        'quizpassword' => '',
        'sumgrades' => 0,
        'grade' => 6,
        'timecreated' => time(),
        'timemodified' => time(),
    ];
    $moduleinfo = clone $quiz;
    $moduleinfo->modulename = 'quiz';
    $moduleinfo->module = $module->id;
    $moduleinfo->section = $section->section;
    $moduleinfo->add = 'quiz';
    $moduleinfo->type = '';
    $moduleinfo->cmidnumber = '';
    $moduleinfo->groupmode = 0;
    $moduleinfo->groupingid = 0;
    $moduleinfo->completion = 0;
    $moduleinfo->availabilityconditionsjson = '{"op":"&","c":[],"showc":[]}';
    $moduleinfo->visible = 1;
    $moduleinfo->visibleoncoursepage = 1;
    $moduleinfo = add_moduleinfo($moduleinfo, $course);
    rebuild_course_cache($course->id, true);
    $created = $DB->get_record('quiz', ['id' => $moduleinfo->instance], '*', MUST_EXIST);
    $created->cmid = $moduleinfo->coursemodule;
    return $created;
}

function create_question_category(stdClass $course): stdClass {
    global $DB;
    $context = context_course::instance($course->id);
    $category = $DB->get_record('question_categories', ['contextid' => $context->id, 'name' => 'moodle-tui local quiz']);
    if ($category) {
        return $category;
    }
    $category = (object)[
        'name' => 'moodle-tui local quiz',
        'contextid' => $context->id,
        'info' => '',
        'infoformat' => FORMAT_HTML,
        'stamp' => make_unique_id_code(),
        'parent' => 0,
        'sortorder' => 999,
        'idnumber' => null,
    ];
    $category->id = $DB->insert_record('question_categories', $category);
    return $category;
}

function save_question(string $qtype, stdClass $form): stdClass {
    global $DB;
    $question = (object)['qtype' => $qtype];
    question_bank::get_qtype($qtype)->save_question($question, $form);
    $records = $DB->get_records('question', ['name' => $form->name], 'id DESC', '*', 0, 1);
    return reset($records);
}

function ensure_question(string $qtype, stdClass $form): stdClass {
    global $DB;
    $existing = $DB->get_records('question', ['name' => $form->name], 'id DESC', '*', 0, 1);
    if ($existing) {
        return reset($existing);
    }
    return save_question($qtype, $form);
}

function recreate_question(string $qtype, stdClass $form): stdClass {
    global $DB;
    $existing = $DB->get_records('question', ['name' => $form->name], 'id DESC', '*', 0, 0);
    foreach ($existing as $question) {
        question_delete_question($question->id);
    }
    return save_question($qtype, $form);
}

function clear_quiz_questions(stdClass $quiz): void {
    global $DB;
    $slots = $DB->get_records('quiz_slots', ['quizid' => $quiz->id], '', 'id');
    foreach ($slots as $slot) {
        $DB->delete_records('question_references', [
            'component' => 'mod_quiz',
            'questionarea' => 'slot',
            'itemid' => $slot->id,
        ]);
    }
    $DB->delete_records('quiz_slots', ['quizid' => $quiz->id]);
    $DB->delete_records('quiz_sections', ['quizid' => $quiz->id]);
}

function add_questions(stdClass $quiz, stdClass $course): void {
    $category = create_question_category($course);
    $common = [
        'category' => $category->id . ',' . $category->contextid,
        'status' => \core_question\local\bank\question_version_status::QUESTION_STATUS_READY,
        'defaultmark' => 1,
        'generalfeedback' => ['text' => '', 'format' => FORMAT_HTML],
        'penalty' => 0.3333333,
    ];

    // ── Q1: True/False (CS) ──────────────────────────────────────────────
    $tf = (object)array_merge($common, [
        'name' => 'CS True False: Time complexity of binary search',
        'questiontext' => [
            'text' => 'The time complexity of binary search on a sorted array of n elements is O(n).',
            'format' => FORMAT_HTML,
        ],
        'correctanswer' => 0,
        'feedbacktrue' => ['text' => 'Binary search is O(log n), not O(n).', 'format' => FORMAT_HTML],
        'feedbackfalse' => ['text' => 'Correct. Binary search halves the search space each iteration, giving O(log n).', 'format' => FORMAT_HTML],
    ]);
    $tfq = ensure_question('truefalse', $tf);

    // ── Q2: Multiple Choice Single (Math) ─────────────────────────────────
    $mc_single = (object)array_merge($common, [
        'name' => 'Math Single: Derivative of x²',
        'questiontext' => [
            'text' => 'What is the derivative of f(x) = x² with respect to x?',
            'format' => FORMAT_HTML,
        ],
        'single' => 1,
        'shuffleanswers' => 1,
        'answernumbering' => 'abc',
        'showstandardinstruction' => 0,
        'noanswers' => 4,
        'numhints' => 0,
        'answer' => [
            ['text' => 'x', 'format' => FORMAT_PLAIN],
            ['text' => '2x', 'format' => FORMAT_PLAIN],
            ['text' => 'x²', 'format' => FORMAT_PLAIN],
            ['text' => '2', 'format' => FORMAT_PLAIN],
        ],
        'fraction' => ['0.0', '1.0', '0.0', '0.0'],
        'feedback' => [
            ['text' => 'Incorrect. The power rule gives 2x.', 'format' => FORMAT_HTML],
            ['text' => 'Correct! d/dx x² = 2x.', 'format' => FORMAT_HTML],
            ['text' => 'Incorrect. The power rule gives 2x.', 'format' => FORMAT_HTML],
            ['text' => 'Incorrect. That would be the derivative of 2x.', 'format' => FORMAT_HTML],
        ],
        'correctfeedback' => ['text' => 'Correct.', 'format' => FORMAT_HTML],
        'partiallycorrectfeedback' => ['text' => '', 'format' => FORMAT_HTML],
        'incorrectfeedback' => ['text' => 'Apply the power rule: bring down the exponent and reduce by one.', 'format' => FORMAT_HTML],
        'shownumcorrect' => 0,
    ]);
    $mc_sinq = recreate_question('multichoice', $mc_single);

    // ── Q3: Multiple Choice Multi (General Knowledge) ─────────────────────
    $mc_multi = (object)array_merge($common, [
        'name' => 'GK Multi: Programming languages',
        'questiontext' => [
            'text' => 'Which of the following are compiled programming languages? (Select all that apply.)',
            'format' => FORMAT_HTML,
        ],
        'single' => 0,
        'shuffleanswers' => 1,
        'answernumbering' => 'abc',
        'showstandardinstruction' => 0,
        'noanswers' => 5,
        'numhints' => 0,
        'answer' => [
            ['text' => 'Rust', 'format' => FORMAT_PLAIN],
            ['text' => 'Python', 'format' => FORMAT_PLAIN],
            ['text' => 'C', 'format' => FORMAT_PLAIN],
            ['text' => 'JavaScript', 'format' => FORMAT_PLAIN],
            ['text' => 'Go', 'format' => FORMAT_PLAIN],
        ],
        'fraction' => ['0.3333333', '0', '0.3333333', '0', '0.3333334'],
        'feedback' => [
            ['text' => 'Correct. Rust is compiled via LLVM.', 'format' => FORMAT_HTML],
            ['text' => 'Python is interpreted (or JIT-compiled at runtime), not traditionally compiled.', 'format' => FORMAT_HTML],
            ['text' => 'Correct. C is compiled to machine code.', 'format' => FORMAT_HTML],
            ['text' => 'JavaScript is interpreted/ JIT-compiled in the browser, not traditionally compiled ahead of time.', 'format' => FORMAT_HTML],
            ['text' => 'Correct. Go is compiled to native binaries.', 'format' => FORMAT_HTML],
        ],
        'correctfeedback' => ['text' => 'Correct! Rust, C, and Go are compiled languages.', 'format' => FORMAT_HTML],
        'partiallycorrectfeedback' => ['text' => 'Some correct, but watch out for the penalties on incorrect selections.', 'format' => FORMAT_HTML],
        'incorrectfeedback' => ['text' => 'Rust, C, and Go are compiled. Python and JavaScript are interpreted at the source level.', 'format' => FORMAT_HTML],
        'shownumcorrect' => 0,
    ]);
    $mc_multiq = recreate_question('multichoice', $mc_multi);

    // ── Q4: Short Answer (History) ────────────────────────────────────────
    $sa = (object)array_merge($common, [
        'name' => 'History Short: Cold War end',
        'questiontext' => [
            'text' => 'In what year did the Berlin Wall fall, marking a key moment in the end of the Cold War?',
            'format' => FORMAT_HTML,
        ],
        'usecase' => 0,
        'answer' => ['1989'],
        'fraction' => [1],
        'feedback' => [['text' => 'Correct. The Berlin Wall fell on 9 November 1989.', 'format' => FORMAT_HTML]],
    ]);
    $saq = ensure_question('shortanswer', $sa);

    // ── Q5: Numerical (Physics) ───────────────────────────────────────────
    $num = (object)array_merge($common, [
        'name' => 'Physics Numerical: Gravitational acceleration',
        'questiontext' => [
            'text' => 'An object is dropped from rest near the Earth\'s surface. Ignoring air resistance, what is its velocity after 3 seconds? (Use g = 9.8 m/s². Enter only the numeric value in m/s.)',
            'format' => FORMAT_HTML,
        ],
        'answer' => ['29.4'],
        'fraction' => [1],
        'tolerance' => [0.1],
        'feedback' => [['text' => 'Correct. v = g × t = 9.8 × 3 = 29.4 m/s.', 'format' => FORMAT_HTML]],
        'unitrole' => 0,
        'unitgradingtypes' => 0,
        'unitpenalty' => 0,
        'showunits' => 0,
        'unitsleft' => 0,
        'unit' => [],
        'multiplier' => [],
    ]);
    $numq = ensure_question('numerical', $num);

    // ── Q6: Multiple Choice Single (CS) ─────────────────────────────────
    $mc_cs = (object)array_merge($common, [
        'name' => 'CS Single: HTTP status for Not Found',
        'questiontext' => [
            'text' => 'Which HTTP status code indicates that a resource was not found on the server?',
            'format' => FORMAT_HTML,
        ],
        'single' => 1,
        'shuffleanswers' => 1,
        'answernumbering' => 'abc',
        'showstandardinstruction' => 0,
        'noanswers' => 4,
        'numhints' => 0,
        'answer' => [
            ['text' => '200', 'format' => FORMAT_PLAIN],
            ['text' => '301', 'format' => FORMAT_PLAIN],
            ['text' => '404', 'format' => FORMAT_PLAIN],
            ['text' => '500', 'format' => FORMAT_PLAIN],
        ],
        'fraction' => ['0.0', '0.0', '1.0', '0.0'],
        'feedback' => [
            ['text' => '200 means OK, not Not Found.', 'format' => FORMAT_HTML],
            ['text' => '301 means Moved Permanently, not Not Found.', 'format' => FORMAT_HTML],
            ['text' => 'Correct! HTTP 404 means Not Found.', 'format' => FORMAT_HTML],
            ['text' => '500 means Internal Server Error.', 'format' => FORMAT_HTML],
        ],
        'correctfeedback' => ['text' => 'Correct.', 'format' => FORMAT_HTML],
        'partiallycorrectfeedback' => ['text' => '', 'format' => FORMAT_HTML],
        'incorrectfeedback' => ['text' => '404 is the standard HTTP status code for "Not Found".', 'format' => FORMAT_HTML],
        'shownumcorrect' => 0,
    ]);
    $mc_csq = recreate_question('multichoice', $mc_cs);

    clear_quiz_questions($quiz);
    foreach ([$tfq, $mc_sinq, $mc_multiq, $saq, $numq, $mc_csq] as $question) {
        quiz_add_quiz_question($question->id, $quiz, 0, 1);
    }
    global $DB;
    if (!$DB->record_exists('quiz_sections', ['quizid' => $quiz->id, 'firstslot' => 1])) {
        $DB->insert_record('quiz_sections', (object)[
            'quizid' => $quiz->id,
            'firstslot' => 1,
            'heading' => '',
            'shufflequestions' => 0,
        ]);
    }
    quiz_update_sumgrades($quiz);
}

function reset_student_quiz_attempts(stdClass $quiz, stdClass $student): void {
    global $DB;
    $quiz->cmid = $quiz->cmid ?? $DB->get_field('course_modules', 'id', [
        'course' => $quiz->course,
        'module' => $DB->get_field('modules', 'id', ['name' => 'quiz']),
        'instance' => $quiz->id,
    ], MUST_EXIST);
    $attempts = $DB->get_records('quiz_attempts', [
        'quiz' => $quiz->id,
        'userid' => $student->id,
        'preview' => 0,
    ]);
    foreach ($attempts as $attempt) {
        quiz_delete_attempt($attempt, $quiz);
    }
}

ensure_mobile_access();
$student = ensure_student();
$course = ensure_course();
ensure_enrolment($course, $student);
$quiz = ensure_quiz($course);
add_questions($quiz, $course);
reset_student_quiz_attempts($quiz, $student);

$trace->output('Local Moodle quiz seed complete.');
$trace->output('URL: http://localhost:8080');
$trace->output('Student: student / studentpass');
$trace->output('Course: TUI-QUIZ');
$trace->output('Quiz: TUI supported questions quiz');
