import { useState } from 'react';
import { AgentSkill, AgentProfile } from '../../../shared/types';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '../ui/card';
import {
  Code,
  Database,
  FileText,
  Shield,
  Settings,
  TestTube,
  Wrench,
  Layout,
  Building,
  Plus,
  X,
} from 'lucide-react';

interface AgentSkillsManagerProps {
  skills: AgentSkill[];
  profiles: AgentProfile[];
  onCreateSkill?: (skill: Partial<AgentSkill>) => void;
  onDeleteSkill?: (skillId: string) => void;
  onCreateProfile?: (profile: Partial<AgentProfile>) => void;
}

const skillIcons: Record<string, React.ReactNode> = {
  frontend: <Layout className="h-4 w-4" />,
  backend: <Code className="h-4 w-4" />,
  database: <Database className="h-4 w-4" />,
  testing: <TestTube className="h-4 w-4" />,
  documentation: <FileText className="h-4 w-4" />,
  security: <Shield className="h-4 w-4" />,
  devops: <Settings className="h-4 w-4" />,
  refactoring: <Wrench className="h-4 w-4" />,
  architecture: <Building className="h-4 w-4" />,
};

const categoryColors: Record<string, string> = {
  development: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
  quality: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
  documentation:
    'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
  security: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300',
  infrastructure:
    'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300',
  architecture:
    'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/30 dark:text-indigo-300',
  general: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300',
};

export function AgentSkillsManager({
  skills,
  profiles,
  onCreateSkill,
  onDeleteSkill,
  onCreateProfile,
}: AgentSkillsManagerProps) {
  const [showNewSkill, setShowNewSkill] = useState(false);
  const [newSkillName, setNewSkillName] = useState('');
  const [newSkillDescription, setNewSkillDescription] = useState('');

  // Group skills by category
  const skillsByCategory = skills.reduce(
    (acc, skill) => {
      const category = skill.category || 'general';
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(skill);
      return acc;
    },
    {} as Record<string, AgentSkill[]>
  );

  const handleCreateSkill = () => {
    if (newSkillName && newSkillDescription && onCreateSkill) {
      onCreateSkill({
        name: newSkillName.toLowerCase().replace(/\s+/g, '_'),
        description: newSkillDescription,
        category: 'general',
      });
      setNewSkillName('');
      setNewSkillDescription('');
      setShowNewSkill(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Skills Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Agent Skills</CardTitle>
              <CardDescription>
                Define skills that agents can have for task assignment
              </CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowNewSkill(!showNewSkill)}
            >
              <Plus className="h-4 w-4 mr-1" />
              Add Skill
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {/* New skill form */}
          {showNewSkill && (
            <div className="mb-4 p-4 border rounded-lg bg-muted/50">
              <div className="space-y-3">
                <Input
                  placeholder="Skill name"
                  value={newSkillName}
                  onChange={(e) => setNewSkillName(e.target.value)}
                />
                <Input
                  placeholder="Description"
                  value={newSkillDescription}
                  onChange={(e) => setNewSkillDescription(e.target.value)}
                />
                <div className="flex gap-2">
                  <Button size="sm" onClick={handleCreateSkill}>
                    Create
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setShowNewSkill(false)}
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            </div>
          )}

          {/* Skills by category */}
          <div className="space-y-4">
            {Object.entries(skillsByCategory).map(
              ([category, categorySkills]) => (
                <div key={category}>
                  <h4 className="text-sm font-medium mb-2 capitalize">
                    {category}
                  </h4>
                  <div className="flex flex-wrap gap-2">
                    {categorySkills.map((skill) => (
                      <Badge
                        key={skill.id}
                        className={`${categoryColors[category] || categoryColors.general} flex items-center gap-1 pr-1`}
                      >
                        {skillIcons[skill.name] || <Code className="h-3 w-3" />}
                        {skill.name}
                        {onDeleteSkill && (
                          <button
                            onClick={() => onDeleteSkill(skill.id)}
                            className="ml-1 hover:bg-black/10 rounded p-0.5"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        )}
                      </Badge>
                    ))}
                  </div>
                </div>
              )
            )}
          </div>
        </CardContent>
      </Card>

      {/* Profiles Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Agent Profiles</CardTitle>
              <CardDescription>
                Configure agent profiles with specific skills and capabilities
              </CardDescription>
            </div>
            {onCreateProfile && (
              <Button variant="outline" size="sm">
                <Plus className="h-4 w-4 mr-1" />
                Add Profile
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {profiles.map((profile) => (
              <div
                key={profile.id}
                className="flex items-center justify-between p-3 border rounded-lg"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{profile.name}</span>
                    {profile.is_planner && (
                      <Badge variant="secondary">Planner</Badge>
                    )}
                    {profile.is_reviewer && (
                      <Badge variant="secondary">Reviewer</Badge>
                    )}
                    {profile.is_worker && (
                      <Badge variant="secondary">Worker</Badge>
                    )}
                  </div>
                  {profile.description && (
                    <p className="text-sm text-muted-foreground mt-1">
                      {profile.description}
                    </p>
                  )}
                </div>
                <div className="text-sm text-muted-foreground">
                  <span className="font-medium">{profile.executor}</span>
                  {profile.variant && <span>:{profile.variant}</span>}
                </div>
              </div>
            ))}

            {profiles.length === 0 && (
              <div className="text-center py-8 text-muted-foreground">
                No agent profiles configured
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
